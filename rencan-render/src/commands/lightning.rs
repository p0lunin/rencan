use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayout, device::Device, pipeline::ComputePipeline,
};

use crate::{
    commands::raw::{
        copy_from_buffer_to_image::CopyFromBufferToImageCommandFactory,
        divide_workgroups::DivideWorkgroupsCommandFactory,
        lights_diffuse::LightsDiffuseCommandFactory,
        reflect_from_mirror::ReflectFromMirrorsCommandFactory,
        trace_rays_to_light::TraceRaysToLightCommandFactory,
    },
    core::{
        intersection::IntersectionUniform, AutoCommandBufferBuilderWrap, CommandFactory,
        CommandFactoryContext, LightRay, Mutable,
    },
};
use once_cell::sync::OnceCell;
use vulkano::{
    buffer::{
        BufferAccess, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, TypedBufferAccess,
    },
    command_buffer::DispatchIndirectCommand,
    descriptor::{
        descriptor_set::{PersistentDescriptorSet, UnsafeDescriptorSetLayout},
        DescriptorSet,
    },
    device::Queue,
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
    sync::GpuFuture,
};

pub mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}

pub struct LightningCommandFactory {
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
    local_size_x: u32,
    sampling: bool,
}

impl LightningCommandFactory {
    pub fn new(device: Arc<Device>, sampling: bool, max_bounces: u32) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = lightning_cs::SpecializationConstants {
            constant_0: local_size_x,
            SAMPLING: if sampling { 1 } else { 0 },
            MAX_BOUNCES: max_bounces,
        };

        let lightning_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &lightning_cs::Shader::load(device).unwrap().main_entry_point(),
                &constants,
                None,
            )
            .unwrap(),
        );
        LightningCommandFactory { lightning_pipeline, local_size_x, sampling }
    }
}

impl CommandFactory for LightningCommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let command = add_lightning(self, &ctx).build();

        Box::new(fut.then_execute(ctx.graphics_queue(), command).unwrap())
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    ctx: &CommandFactoryContext,
) -> AutoCommandBufferBuilderWrap {
    let CommandFactoryContext { buffers, .. } = ctx;

    let set_0 = buffers.global_app_set.clone();
    let set_1 = buffers.intersections_set.clone();
    let set_2 = buffers.models_set.clone();
    let set_3 = buffers.sphere_models_set.clone();
    let set_4 = buffers.lights_set.clone();
    let set_5 = buffers.image_set.clone();

    match factory.sampling {
        true => ctx
            .create_command_buffer()
            .dispatch(
                ctx.app_info.size_of_image_array() as u32 / factory.local_size_x,
                factory.lightning_pipeline.clone(),
                (set_0, set_1, set_2, set_3, set_4, set_5),
            )
            .unwrap(),
        false => ctx.create_command_buffer().dispatch_indirect(
            buffers.workgroups.clone(),
            factory.lightning_pipeline.clone(),
            (set_0, set_1, set_2, set_3, set_4, set_5),
        ),
    }
}

pub struct LightningV2CommandFactory {
    divide_factory: DivideWorkgroupsCommandFactory,
    trace_rays_factory: TraceRaysToLightCommandFactory,
    lights_factory: LightsDiffuseCommandFactory,
    trace_mirrors_factory: ReflectFromMirrorsCommandFactory,
    copy_factory: CopyFromBufferToImageCommandFactory,
    intersections_set: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[LightRay]>>>>,
    reflects_intersections_set:
        Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[IntersectionUniform]>>>>,
    image_buffer_set: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[[u32; 4]]>>>>,
    workgroups: OnceCell<[OneBufferSet<Arc<DeviceLocalBuffer<[DispatchIndirectCommand]>>>; 2]>,
}

impl LightningV2CommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);
        LightningV2CommandFactory {
            divide_factory: DivideWorkgroupsCommandFactory::new(device.clone(), local_size_x),
            trace_rays_factory: TraceRaysToLightCommandFactory::new(device.clone()),
            lights_factory: LightsDiffuseCommandFactory::new(device.clone()),
            trace_mirrors_factory: ReflectFromMirrorsCommandFactory::new(device.clone()),
            copy_factory: CopyFromBufferToImageCommandFactory::new(device.clone()),
            intersections_set: Mutable::new(0),
            reflects_intersections_set: Mutable::new(0),
            image_buffer_set: Mutable::new(0),
            workgroups: OnceCell::new(),
        }
    }
    fn init_reflects_intersections_set(
        &mut self,
        ctx: &CommandFactoryContext,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.reflects_intersections_set.change_with_check_in_place(ctx.buffers.intersections.len());
        let layout = self.trace_mirrors_factory.intersections_layout();
        let one_set = self
            .reflects_intersections_set
            .get_depends_or_init(|&len| {
                let buf = ctx.create_device_local_buffer_array(
                    len,
                    BufferUsage {
                        storage_buffer: true,
                        transfer_destination: true,
                        ..BufferUsage::none()
                    },
                );
                let set = OneBufferSet::new(buf, layout);
                set
            })
            .clone();
        one_set.1.clone()
    }
    fn init_intersections_set(
        &mut self,
        ctx: &CommandFactoryContext,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.intersections_set.change_with_check_in_place(
            ctx.buffers.intersections.len() * (ctx.scene.data.point_lights.len() + 1),
        );
        let layout = self.trace_rays_factory.intersections_layout();
        let one_set = self
            .intersections_set
            .get_depends_or_init(|&len| {
                let buf = ctx.create_device_local_buffer_array(
                    len,
                    BufferUsage {
                        storage_buffer: true,
                        transfer_destination: true,
                        ..BufferUsage::none()
                    },
                );
                let set = OneBufferSet::new(buf, layout);
                set
            })
            .clone();
        one_set.1.clone()
    }
    fn init_image_buffer_set(
        &mut self,
        ctx: &CommandFactoryContext,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.image_buffer_set.change_with_check_in_place(ctx.app_info.size_of_image_array());
        let layout = self.trace_rays_factory.intersections_layout();
        let one_set = self
            .image_buffer_set
            .get_depends_or_init(|&len| {
                let buf = ctx.create_device_local_buffer_array(
                    len,
                    BufferUsage {
                        storage_buffer: true,
                        transfer_destination: true,
                        ..BufferUsage::none()
                    },
                );
                let set = OneBufferSet::new(buf, layout);
                set
            })
            .clone();
        one_set.1.clone()
    }
    fn init_workgroups_set(
        &self,
        ctx: &CommandFactoryContext,
    ) -> [OneBufferSet<Arc<DeviceLocalBuffer<[DispatchIndirectCommand]>>>; 2] {
        match self.workgroups.get() {
            None => {
                let buf = ctx.create_device_local_buffer_array(
                    1,
                    BufferUsage {
                        storage_buffer: true,
                        indirect_buffer: true,
                        transfer_destination: true,
                        ..BufferUsage::none()
                    },
                );
                let set = OneBufferSet::new(buf, self.trace_rays_factory.rays_layout());

                let buf2 = ctx.create_device_local_buffer_array(
                    1,
                    BufferUsage {
                        storage_buffer: true,
                        indirect_buffer: true,
                        transfer_destination: true,
                        ..BufferUsage::none()
                    },
                );
                let set2 = OneBufferSet::new(buf2, self.trace_rays_factory.rays_layout());

                self.workgroups
                    .set([set.clone(), set2.clone()])
                    .unwrap_or_else(|_| unreachable!("we already check that it None"));

                [set.clone(), set2.clone()]
            }
            Some(set) => set.clone(),
        }
    }
}

impl CommandFactory for LightningV2CommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let intersections_set = self.init_intersections_set(&ctx);
        let reflects_set = self.init_reflects_intersections_set(&ctx);
        let image_buffer_set = self.init_image_buffer_set(&ctx);
        let [workgroups_set1, workgroups_set2] = self.init_workgroups_set(&ctx);

        debug_assert_eq!(workgroups_set2.0.len(), 1);

        let cmd_zeroes = ctx
            .create_command_buffer()
            .update_with(|buf| {
                buf.0.fill_buffer(workgroups_set1.0.clone(), 0).unwrap();
                buf.0.fill_buffer(workgroups_set2.0.clone(), 0).unwrap();
                buf.0
                    .fill_buffer(
                        self.intersections_set.get_depends_or_init(|_| unreachable!()).0.clone(),
                        0,
                    )
                    .unwrap();
                buf.0
                    .fill_buffer(
                        self.reflects_intersections_set
                            .get_depends_or_init(|_| unreachable!())
                            .0
                            .clone(),
                        0,
                    )
                    .unwrap();
                buf.0
                    .fill_buffer(
                        self.image_buffer_set.get_depends_or_init(|_| unreachable!()).0.clone(),
                        0,
                    )
                    .unwrap();
            })
            .build();

        let cmd_1_trace_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_rays_factory.add_trace_rays_to_buffer(
                    &ctx,
                    ctx.buffers.workgroups.clone(),
                    ctx.buffers.intersections_set.clone(),
                    intersections_set.clone(),
                    workgroups_set2.1.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_2_divide = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.divide_factory.add_divider_to_buffer(workgroups_set2.1.clone(), &mut buf.0);
            })
            .build();

        let cmd_3_lights_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.lights_factory.add_lights_diffuse_to_buffer(
                    &ctx,
                    workgroups_set2.0.clone(),
                    intersections_set.clone(),
                    ctx.buffers.intersections_set.clone(),
                    image_buffer_set.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_4_trace_mirrors = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_mirrors_factory.add_reflects_rays_to_buffer(
                    &ctx,
                    ctx.buffers.workgroups.clone(),
                    ctx.buffers.intersections_set.clone(),
                    reflects_set.clone(),
                    workgroups_set1.1.clone(),
                    &mut buf.0,
                );
                buf.0.fill_buffer(workgroups_set2.0.clone(), 0).unwrap();
            })
            .build();

        let cmd_5_divide = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.divide_factory.add_divider_to_buffer(workgroups_set1.1.clone(), &mut buf.0);
            })
            .build();

        let cmd_6_trace_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_rays_factory.add_trace_rays_to_buffer(
                    &ctx,
                    workgroups_set1.0.clone(),
                    reflects_set.clone(),
                    intersections_set.clone(),
                    workgroups_set2.1.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_7_divide = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.divide_factory.add_divider_to_buffer(workgroups_set2.1.clone(), &mut buf.0);
            })
            .build();

        let cmd_8_lights_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.lights_factory.add_lights_diffuse_to_buffer(
                    &ctx,
                    workgroups_set2.0.clone(),
                    intersections_set.clone(),
                    reflects_set,
                    image_buffer_set.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_10_copy_command = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.copy_factory.add_copy(&ctx, image_buffer_set.clone(), &mut buf.0)
            })
            .build();

        fut.then_execute(ctx.graphics_queue(), cmd_zeroes)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd_1_trace_diffuse)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd_2_divide)
            .unwrap()
            .then_signal_semaphore() // vulkano does not provide barrier for this
            .then_execute(ctx.graphics_queue(), cmd_3_lights_diffuse)
            .unwrap()
            .then_signal_semaphore()
            .then_execute(ctx.graphics_queue(), cmd_4_trace_mirrors)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd_5_divide)
            .unwrap()
            .then_signal_semaphore()
            .then_execute(ctx.graphics_queue(), cmd_6_trace_diffuse)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd_7_divide)
            .unwrap()
            .then_signal_semaphore()
            .then_execute(ctx.graphics_queue(), cmd_8_lights_diffuse)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd_10_copy_command)
            .unwrap()
            .boxed()
    }
}

#[derive(Clone)]
struct OneBufferSet<Buf>(Buf, Arc<dyn DescriptorSet + Send + Sync>);

impl<Buf> OneBufferSet<Buf>
where
    Buf: BufferAccess + Clone + Send + Sync + 'static,
{
    fn new(buf: Buf, layout: Arc<UnsafeDescriptorSetLayout>) -> Self {
        let set = PersistentDescriptorSet::start(layout)
            .add_buffer(buf.clone())
            .unwrap()
            .build()
            .unwrap();
        Self(buf, Arc::new(set))
    }
}
