use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayout, device::Device, pipeline::ComputePipeline,
};

use crate::core::{AutoCommandBufferBuilderWrap, CommandFactory, CommandFactoryContext, LightRay, Mutable};
use vulkano::sync::GpuFuture;
use crate::commands::raw::make_lightning_rays_diffuse::MakeLightningRaysDiffuseCommandFactory;
use crate::commands::raw::trace_rays_to_light::TraceRaysToLightCommandFactory;
use crate::commands::raw::lights_diffuse::LightsDiffuseCommandFactory;
use vulkano::buffer::{CpuAccessibleBuffer, TypedBufferAccess, DeviceLocalBuffer, BufferUsage, BufferAccess};
use vulkano::descriptor::DescriptorSet;
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet, UnsafeDescriptorSetLayout};
use vulkano::command_buffer::DispatchIndirectCommand;
use vulkano::device::Queue;
use once_cell::sync::OnceCell;
use vulkano::memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc};
use crate::core::intersection::LightIntersection;
use crate::commands::raw::divide_workgroups::DivideWorkgroupsCommandFactory;

pub mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}

pub struct LightningCommandFactory {
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
    local_size_x: u32,
    sampling: bool
}

impl LightningCommandFactory {
    pub fn new(device: Arc<Device>, sampling: bool, max_bounces: u32,) -> Self {
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
        true => {
            ctx.create_command_buffer().dispatch(
                ctx.app_info.size_of_image_array() as u32 / factory.local_size_x,
                factory.lightning_pipeline.clone(),
                (set_0, set_1, set_2, set_3, set_4, set_5),
            ).unwrap()
        }
        false => {
            ctx.create_command_buffer().dispatch_indirect(
                buffers.workgroups.clone(),
                factory.lightning_pipeline.clone(),
                (set_0, set_1, set_2, set_3, set_4, set_5),
            )
        }
    }
}

pub struct LightningV2CommandFactory {
    make_rays_factory: MakeLightningRaysDiffuseCommandFactory,
    divide_factory: DivideWorkgroupsCommandFactory,
    trace_rays_factory: TraceRaysToLightCommandFactory,
    lights_factory: LightsDiffuseCommandFactory,
    light_rays: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[LightRay]>>>>,
    intersections_set: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[LightIntersection]>>>>,
    workgroups: OnceCell<[OneBufferSet<Arc<DeviceLocalBuffer<[DispatchIndirectCommand]>>>; 2]>
}

impl LightningV2CommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x =
            device.physical_device().extended_properties().subgroup_size().unwrap_or(32);
        LightningV2CommandFactory {
            make_rays_factory: MakeLightningRaysDiffuseCommandFactory::new(device.clone()),
            divide_factory: DivideWorkgroupsCommandFactory::new(device.clone(), local_size_x),
            trace_rays_factory: TraceRaysToLightCommandFactory::new(device.clone()),
            lights_factory: LightsDiffuseCommandFactory::new(device.clone()),
            light_rays: Mutable::new(0),
            intersections_set: Mutable::new(0),
            workgroups: OnceCell::new(),
        }
    }
    fn init_rays_set(&mut self, ctx: &CommandFactoryContext) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.light_rays.change_with_check_in_place(ctx.buffers.intersections.len());
        let layout = self.trace_rays_factory.rays_layout();
        let one_set = self.light_rays.get_depends_or_init(|&len| {
            let buf = ctx.create_device_local_buffer_array(
                len,
                BufferUsage {  storage_buffer: true, ..BufferUsage::none() },
            );
            let set = OneBufferSet::new(
                buf,
                layout
            );
            set
        });
        one_set.1.clone()
    }
    fn init_intersections_set(&mut self, ctx: &CommandFactoryContext) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.intersections_set.change_with_check_in_place(ctx.buffers.intersections.len());
        let layout = self.trace_rays_factory.intersections_layout();
        let one_set = self.intersections_set.get_depends_or_init(|&len| {
            let buf = ctx.create_device_local_buffer_array(
                len,
                BufferUsage {  storage_buffer: true, ..BufferUsage::none() },
            );
            let set = OneBufferSet::new(
                buf,
                layout
            );
            set
        }).clone();
        one_set.1.clone()
    }
    fn init_workgroups_set(&self, ctx: &CommandFactoryContext) -> [OneBufferSet<Arc<DeviceLocalBuffer<[DispatchIndirectCommand]>>>; 2] {
        match self.workgroups.get() {
            None => {
                let buf = ctx.create_device_local_buffer_array(
                    1,
                    BufferUsage { storage_buffer: true, indirect_buffer: true, transfer_destination: true, ..BufferUsage::none() },
                );
                let set = OneBufferSet::new(
                    buf,
                    self.trace_rays_factory.rays_layout()
                );

                let buf2 = ctx.create_device_local_buffer_array(
                    1,
                    BufferUsage { storage_buffer: true, indirect_buffer: true, transfer_destination: true, ..BufferUsage::none() },
                );
                let set2 = OneBufferSet::new(
                    buf2,
                    self.trace_rays_factory.rays_layout()
                );

                self.workgroups.set([set.clone(), set2.clone()]).unwrap_or_else(|_| unreachable!("we already check that it None"));

                [set.clone(), set2.clone()]
            }
            Some(set) => set.clone()
        }
    }
}

impl CommandFactory for LightningV2CommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let mut cmd_zeroes = ctx.create_command_buffer();
        let mut cmd1 = ctx.create_command_buffer();
        let mut cmd3 = ctx.create_command_buffer();
        let mut cmd5 = ctx.create_command_buffer();

        let rays_set = self.init_rays_set(&ctx);
        let intersections_set = self.init_intersections_set(&ctx);
        let [workgroups_set1, workgroups_set2] = self.init_workgroups_set(&ctx);

        debug_assert_eq!(workgroups_set1.0.len(), 1);
        debug_assert_eq!(workgroups_set2.0.len(), 1);

        cmd_zeroes.0.fill_buffer(workgroups_set1.0.clone(), 0).unwrap();
        cmd_zeroes.0.fill_buffer(workgroups_set2.0.clone(), 0).unwrap();

        let cmd_zeroes = cmd_zeroes.build();

        self.make_rays_factory.add_making_rays_to_buffer(
            &ctx,
            ctx.buffers.workgroups.clone(),
            rays_set.clone(),
            ctx.buffers.intersections_set.clone(),
            workgroups_set1.1.clone(),
            &mut cmd1.0,
        );

        let mut cmd2 = ctx.create_command_buffer();
        self.divide_factory.add_divider_to_buffer(
            workgroups_set1.1.clone(),
            &mut cmd2.0
        );

        self.trace_rays_factory.add_trace_rays_to_buffer(
            &ctx,
            workgroups_set1.0.clone(),
            rays_set.clone(),
            intersections_set.clone(),
            ctx.buffers.intersections_set.clone(),
            workgroups_set2.1.clone(),
            &mut cmd3.0
        );

        let mut cmd4 = ctx.create_command_buffer();
        self.divide_factory.add_divider_to_buffer(
            workgroups_set2.1.clone(),
            &mut cmd4.0
        );

        self.lights_factory.add_lights_diffuse_to_buffer(
            &ctx,
            workgroups_set2.0.clone(),
            intersections_set.clone(),
            ctx.buffers.image_set.clone(),
            &mut cmd5.0
        );

        let cmd1 = cmd1.build();
        let cmd2 = cmd2.build();
        let cmd3 = cmd3.build();
        let cmd4 = cmd4.build();
        let cmd5 = cmd5.build();

        fut
            .then_execute(ctx.graphics_queue(), cmd_zeroes)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd1)
            .unwrap()
        .then_execute(ctx.graphics_queue(), cmd2)
            .unwrap()
        .then_execute(ctx.graphics_queue(), cmd3)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd4)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd5)
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
