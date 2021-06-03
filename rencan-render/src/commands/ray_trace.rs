use std::sync::Arc;

use vulkano::{
    descriptor::pipeline_layout::PipelineLayout, device::Device, pipeline::ComputePipeline,
};

use crate::{
    commands::raw::divide_workgroups::DivideWorkgroupsCommandFactory,
    core::{camera::Camera, AppInfo, CommandFactory, CommandFactoryContext, Screen},
};
use nalgebra::Point3;
use vulkano::{format::ClearValue, sync::GpuFuture};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/ray_tracing.glsl"
    }
}

pub mod ray_trace_shader {
    pub use super::cs::*;
}

pub struct InitialTraceCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::MainLayout>>>,
    divide_factory: DivideWorkgroupsCommandFactory,
    local_size_x: u32,
}

impl InitialTraceCommandFactory {
    pub fn new(info: &AppInfo) -> Self {
        let device = &info.device;
        let shader = cs::Shader::load(device.clone()).unwrap();
        let local_size_x = info.recommend_workgroups_length;

        let constants = cs::SpecializationConstants { constant_0: local_size_x };

        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        let divide_factory = DivideWorkgroupsCommandFactory::new(device.clone(), local_size_x);
        InitialTraceCommandFactory {
            pipeline,
            divide_factory,
            local_size_x,
        }
    }
}

impl CommandFactory for InitialTraceCommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        fut: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let buffers = &ctx.buffers;

        let set_0 = buffers.global_app_set.clone();
        let set_1 = buffers.intersections_set.clone();
        let set_2 = buffers.models_set.clone();
        let set_3 = buffers.sphere_models_set.clone();
        let set_4 = buffers.lights_set.clone();
        let set_5 = buffers.workgroups_set.clone();
        let set_6 = buffers.image_set.clone();

        let sets = (set_0, set_1, set_2, set_3, set_4, set_5, set_6);

        let ray_trace_command = ctx
            .create_command_buffer()
            .update_with(|buf| {
                buf.0
                    .dispatch(
                        [ctx.app_info.size_of_image_array() as u32 / self.local_size_x, 1, 1],
                        self.pipeline.clone(),
                        sets,
                        (
                            ctx.app_info.size_of_image_array() as u32 * ctx.render_step,
                            ctx.app_info.msaa as u32,
                        ),
                        std::iter::empty(),
                    )
                    .unwrap();
            })
            .build();

        let mut divide_command = ctx.create_command_buffer();
        self.divide_factory
            .add_divider_to_buffer(buffers.workgroups_set.clone(), &mut divide_command.0);
        let divide_command = divide_command.build();

        fut.then_execute(ctx.graphics_queue(), ray_trace_command)
            .unwrap()
            .then_execute(ctx.graphics_queue(), divide_command)
            .unwrap()
            .boxed()
    }
}
