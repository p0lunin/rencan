use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayout, device::Device, pipeline::ComputePipeline,
};

use crate::{
    commands::raw::{
        copy_from_buffer_to_image::CopyFromBufferToImageCommandFactory,
        divide_workgroups::DivideWorkgroupsCommandFactory,
        lights_diffuse::LightsDiffuseCommandFactory, lights_gi::LightsGiCommandFactory,
        make_gi_rays::MakeGiRaysCommandFactory,
        reflect_from_mirror::ReflectFromMirrorsCommandFactory,
        trace_rays_to_light::TraceRaysToLightCommandFactory,
    },
    core::{
        intersection::IntersectionUniform, AppInfo, AutoCommandBufferBuilderWrap, CommandFactory,
        CommandFactoryContext, LightRay, Mutable,
    },
};
use once_cell::sync::OnceCell;
use vulkano::{
    buffer::{BufferAccess, BufferUsage, DeviceLocalBuffer, TypedBufferAccess},
    command_buffer::DispatchIndirectCommand,
    descriptor::{
        descriptor_set::{PersistentDescriptorSet, UnsafeDescriptorSetLayout},
        DescriptorSet,
    },
    sync::GpuFuture,
};

pub struct GiCommandFactory {
    divide_factory: DivideWorkgroupsCommandFactory,
    trace_rays_factory: TraceRaysToLightCommandFactory,
    lights_factory: LightsDiffuseCommandFactory,
    trace_mirrors_factory: ReflectFromMirrorsCommandFactory,
    copy_factory: CopyFromBufferToImageCommandFactory,
    make_gi_rays_factory: MakeGiRaysCommandFactory,
    lights_gi_factory: LightsGiCommandFactory,
    intersections_set: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[LightRay]>>>>,
    image_buffer_set: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[[u32; 4]]>>>>,
    gi_intersections_set:
        Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[IntersectionUniform]>>>>,
    gi_thetas_set: Mutable<usize, OneBufferSet<Arc<DeviceLocalBuffer<[[f32; 4]]>>>>,
    workgroups: OnceCell<[OneBufferSet<Arc<DeviceLocalBuffer<[DispatchIndirectCommand]>>>; 3]>,
    samples_per_bounce: u32,
}

impl GiCommandFactory {
    pub fn new(info: &AppInfo, samples_per_bounce: u32) -> Self {
        let device = &info.device;
        let local_size_x = info.recommend_workgroups_length;

        GiCommandFactory {
            divide_factory: DivideWorkgroupsCommandFactory::new(device.clone(), local_size_x),
            trace_rays_factory: TraceRaysToLightCommandFactory::new(info),
            lights_factory: LightsDiffuseCommandFactory::new(info),
            trace_mirrors_factory: ReflectFromMirrorsCommandFactory::new(info),
            copy_factory: CopyFromBufferToImageCommandFactory::new(info),
            make_gi_rays_factory: MakeGiRaysCommandFactory::new(info),
            lights_gi_factory: LightsGiCommandFactory::new(info, samples_per_bounce),
            intersections_set: Mutable::new(0),
            image_buffer_set: Mutable::new(0),
            gi_intersections_set: Mutable::new(0),
            gi_thetas_set: Mutable::new(0),
            workgroups: OnceCell::new(),
            samples_per_bounce,
        }
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
    fn init_gi_intersections_set(
        &mut self,
        ctx: &CommandFactoryContext,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.gi_intersections_set.change_with_check_in_place(ctx.buffers.intersections.len());
        let layout = self.make_gi_rays_factory.intersections_layout();
        let one_set = self
            .gi_intersections_set
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
    fn init_gi_thetas_set(
        &mut self,
        ctx: &CommandFactoryContext,
    ) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.gi_thetas_set.change_with_check_in_place(ctx.buffers.intersections.len());
        let layout = self.make_gi_rays_factory.gi_thetas_set();
        let one_set = self
            .gi_thetas_set
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
    ) -> [OneBufferSet<Arc<DeviceLocalBuffer<[DispatchIndirectCommand]>>>; 3] {
        match self.workgroups.get() {
            None => {
                let init_buf_and_set = || {
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
                    set
                };
                let set1 = init_buf_and_set();
                let set2 = init_buf_and_set();
                let set3 = init_buf_and_set();
                self.workgroups
                    .set([set1.clone(), set2.clone(), set3.clone()])
                    .unwrap_or_else(|_| unreachable!("we already check that it None"));

                [set1.clone(), set2.clone(), set3.clone()]
            }
            Some(set) => set.clone(),
        }
    }
}

impl GiCommandFactory {
    fn add_lightning_commands<WB1, WB2, WS2, PIS, TIS, ImS>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_buf1: WB1,
        workgroups_buf2: WB2,
        workgroups_set2: WS2,
        previous_inters_set: PIS,
        temp_inters_set: TIS,
        image_set: ImS,
        fut: impl GpuFuture + 'static,
    ) -> Box<dyn GpuFuture>
    where
        WB1: TypedBufferAccess<Content = [DispatchIndirectCommand]> + Clone + Send + Sync + 'static,
        WB2: TypedBufferAccess<Content = [DispatchIndirectCommand]> + Clone + Send + Sync + 'static,
        WS2: DescriptorSet + Clone + Send + Sync + 'static,
        PIS: DescriptorSet + Clone + Send + Sync + 'static,
        TIS: DescriptorSet + Clone + Send + Sync + 'static,
        ImS: DescriptorSet + Clone + Send + Sync + 'static,
    {
        let cmd_1_trace_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_rays_factory.add_trace_rays_to_buffer(
                    &ctx,
                    workgroups_buf1.clone(),
                    previous_inters_set.clone(),
                    temp_inters_set.clone(),
                    workgroups_set2.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_trace_mirrors = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_mirrors_factory.add_reflects_rays_to_buffer(
                    &ctx,
                    ctx.buffers.workgroups.clone(),
                    ctx.buffers.intersections_set.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let fut = fut
            .then_execute(ctx.graphics_queue(), cmd_1_trace_diffuse)
            .unwrap()
            .then_execute_same_queue(cmd_trace_mirrors)
            .unwrap();

        let cmd_2_divide = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.divide_factory.add_divider_to_buffer(workgroups_set2.clone(), &mut buf.0);
            })
            .build();

        let cmd_3_lights_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.lights_factory.add_lights_diffuse_to_buffer(
                    &ctx,
                    workgroups_buf2.clone(),
                    temp_inters_set.clone(),
                    previous_inters_set.clone(),
                    image_set.clone(),
                    &mut buf.0,
                );
            })
            .build();

        fut.then_execute(ctx.graphics_queue(), cmd_2_divide)
            .unwrap()
            .then_signal_semaphore() // vulkano does not provide barrier for this
            .then_execute(ctx.graphics_queue(), cmd_3_lights_diffuse)
            .unwrap()
            .boxed()
    }

    fn add_reflects_pipeline<WS1, RS>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_set1: WS1,
        fut: impl GpuFuture + 'static,
    ) -> Box<dyn GpuFuture>
    where
        WS1: DescriptorSet + Clone + Send + Sync + 'static,
        RS: DescriptorSet + Clone + Send + Sync + 'static,
    {
        let cmd_trace_mirrors = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_mirrors_factory.add_reflects_rays_to_buffer(
                    &ctx,
                    ctx.buffers.workgroups.clone(),
                    ctx.buffers.intersections_set.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_divide_for_trace = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.divide_factory.add_divider_to_buffer(workgroups_set1.clone(), &mut buf.0);
            })
            .build();

        let fut = fut
            .then_execute_same_queue(cmd_trace_mirrors)
            .unwrap()
            .then_execute_same_queue(cmd_divide_for_trace)
            .unwrap()
            .then_signal_semaphore();

        fut.boxed()
    }

    fn add_gi_lightning_commands<WB1, WB2, WS2, PIS, TIS, ImS, GTS>(
        &self,
        ctx: &CommandFactoryContext,
        workgroups_buf1: WB1,
        workgroups_buf2: WB2,
        workgroups_set2: WS2,
        previous_inters_set: PIS,
        temp_inters_set: TIS,
        image_set: ImS,
        gi_thetas_set: GTS,
        fut: impl GpuFuture + 'static,
    ) -> Box<dyn GpuFuture>
    where
        WB1: TypedBufferAccess<Content = [DispatchIndirectCommand]> + Clone + Send + Sync + 'static,
        WB2: TypedBufferAccess<Content = [DispatchIndirectCommand]> + Clone + Send + Sync + 'static,
        WS2: DescriptorSet + Clone + Send + Sync + 'static,
        PIS: DescriptorSet + Clone + Send + Sync + 'static,
        TIS: DescriptorSet + Clone + Send + Sync + 'static,
        ImS: DescriptorSet + Clone + Send + Sync + 'static,
        GTS: DescriptorSet + Clone + Send + Sync + 'static,
    {
        let cmd_1_trace_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.trace_rays_factory.add_trace_rays_to_buffer(
                    &ctx,
                    workgroups_buf1.clone(),
                    previous_inters_set.clone(),
                    temp_inters_set.clone(),
                    workgroups_set2.clone(),
                    &mut buf.0,
                );
            })
            .build();

        let cmd_2_divide = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.divide_factory.add_divider_to_buffer(workgroups_set2.clone(), &mut buf.0);
            })
            .build();

        let cmd_3_lights_diffuse = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.lights_gi_factory.add_lights_diffuse_to_buffer(
                    &ctx,
                    workgroups_buf2.clone(),
                    temp_inters_set.clone(),
                    previous_inters_set.clone(),
                    gi_thetas_set.clone(),
                    image_set.clone(),
                    &mut buf.0,
                );
            })
            .build();

        fut.then_execute(ctx.graphics_queue(), cmd_1_trace_diffuse)
            .unwrap()
            .then_execute(ctx.graphics_queue(), cmd_2_divide)
            .unwrap()
            .then_signal_semaphore() // vulkano does not provide barrier for this
            .then_execute(ctx.graphics_queue(), cmd_3_lights_diffuse)
            .unwrap()
            .boxed()
    }
}

impl CommandFactory for GiCommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let intersections_set = self.init_intersections_set(&ctx);
        let image_buffer_set = self.init_image_buffer_set(&ctx);
        let gi_intersections_set = self.init_gi_intersections_set(&ctx);
        let gi_thetas_set = self.init_gi_thetas_set(&ctx);
        let [workgroups_set1, workgroups_set2, workgroups_set3] = self.init_workgroups_set(&ctx);

        debug_assert_eq!(workgroups_set2.0.len(), 1);

        let cmd_zeroes = ctx
            .create_command_buffer()
            .update_with(|buf| {
                buf.0.fill_buffer(workgroups_set1.0.clone(), 0).unwrap();
                buf.0.fill_buffer(workgroups_set2.0.clone(), 0).unwrap();
                buf.0.fill_buffer(workgroups_set3.0.clone(), 0).unwrap();
                buf.0
                    .fill_buffer(
                        self.image_buffer_set.get_depends_or_init(|_| unreachable!()).0.clone(),
                        0,
                    )
                    .unwrap();
            })
            .build();

        let fut = fut.then_execute(ctx.app_info.graphics_queue.clone(), cmd_zeroes).unwrap();
        let fut = self.add_lightning_commands(
            &ctx,
            ctx.buffers.workgroups.clone(),
            workgroups_set2.0.clone(),
            workgroups_set2.1.clone(),
            ctx.buffers.intersections_set.clone(),
            intersections_set.clone(),
            image_buffer_set.clone(),
            fut,
        );

        fut.flush().unwrap();

        let mut fut = fut.boxed();
        for sample_number in 0..self.samples_per_bounce {
            let cmd_zeros = ctx
                .create_command_buffer()
                .update_with(|buf| {
                    buf.0.fill_buffer(workgroups_set1.0.clone(), 0).unwrap();
                    buf.0.fill_buffer(workgroups_set3.0.clone(), 0).unwrap();
                })
                .build();
            let cmd_make_gi_rays = ctx
                .create_command_buffer()
                .update_with(|buf| {
                    self.make_gi_rays_factory.add_making_gi_rays(
                        sample_number,
                        &ctx,
                        ctx.buffers.workgroups.clone(),
                        ctx.buffers.intersections_set.clone(),
                        gi_intersections_set.clone(),
                        workgroups_set3.1.clone(),
                        gi_thetas_set.clone(),
                        &mut buf.0,
                    );
                })
                .build();

            let cmd_divide = ctx
                .create_command_buffer()
                .update_with(|buf| {
                    self.divide_factory
                        .add_divider_to_buffer(workgroups_set3.1.clone(), &mut buf.0);
                    buf.0.fill_buffer(workgroups_set1.0.clone(), 0).unwrap();
                })
                .build();

            let this_fut = fut
                .then_execute_same_queue(cmd_zeros)
                .unwrap()
                .then_execute_same_queue(cmd_make_gi_rays)
                .unwrap()
                .then_execute_same_queue(cmd_divide)
                .unwrap()
                .then_signal_semaphore()
                .boxed();
            fut = self.add_gi_lightning_commands(
                &ctx,
                workgroups_set3.0.clone(),
                workgroups_set1.0.clone(),
                workgroups_set1.1.clone(),
                gi_intersections_set.clone(),
                intersections_set.clone(),
                image_buffer_set.clone(),
                gi_thetas_set.clone(),
                this_fut,
            );
            fut.flush().unwrap();
        }

        let cmd_last_copy_command = ctx
            .create_command_buffer()
            .update_with(|buf| {
                self.copy_factory.add_copy(&ctx, image_buffer_set.clone(), &mut buf.0)
            })
            .build();

        fut.then_execute(ctx.graphics_queue(), cmd_last_copy_command).unwrap().boxed()
    }
}

#[derive(Clone)]
pub struct OneBufferSet<Buf>(pub Buf, pub Arc<dyn DescriptorSet + Send + Sync>);

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
