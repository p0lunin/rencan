use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::pipeline_layout::PipelineLayout,
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext, Screen};
use std::cell::RefCell;
use crate::core::camera::Camera;
use nalgebra::Point3;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::PipelineLayoutAbstract;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/ray_tracing.glsl"
    }
}

mod divide_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/divide_workgroup_size.glsl"
    }
}

pub mod ray_trace_shader {
    pub use super::cs::*;
}

pub struct RayTraceCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
    divide_pipeline: Arc<ComputePipeline<PipelineLayout<divide_cs::Layout>>>,
    prev_camera: Camera,
    prev_screen: Screen,
    local_size_x: u32,
}

impl RayTraceCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let divide_shader = divide_cs::Shader::load(device.clone()).unwrap();
        let local_size_x = device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = cs::SpecializationConstants {
            constant_0: local_size_x,
        };
        let divide_constants = divide_cs::SpecializationConstants {
            DIVIDER: local_size_x,
        };

        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None).unwrap(),
        );
        let divide_pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &divide_shader.main_entry_point(), &divide_constants, None).unwrap(),
        );
        RayTraceCommandFactory {
            pipeline,
            divide_pipeline,
            prev_camera: Camera::new(
                Point3::new(f32::NAN, f32::NAN, f32::NAN),
                (f32::NAN, f32::NAN, f32::NAN),
                0.0,
            ),
            prev_screen: Screen::new(0, 0),
            local_size_x,
        }
    }
}

impl CommandFactory for RayTraceCommandFactory {
    fn make_command(&mut self, ctx: CommandFactoryContext, commands: &mut Vec<AutoCommandBuffer>,
    )  {
        /*if self.prev_screen == ctx.app_info.screen
            && self.prev_camera == *ctx.camera
        {
            return;
        }*/

        self.prev_camera = ctx.camera.clone();
        self.prev_screen = ctx.app_info.screen.clone();

        let buffers = &ctx.buffers;

        let set_0 = buffers.global_app_set.clone();
        let set_1 = buffers.rays_set.clone();
        let set_2 = buffers.models_set.clone();

        let sets = (set_0, set_1, set_2);

        let command = ctx
            .create_command_buffer()
            .dispatch(ctx.app_info.size_of_image_array() as u32 / self.local_size_x, self.pipeline.clone(), sets)
            .unwrap()
            .build();

        commands.push(command);

        let set = Arc::new(
            PersistentDescriptorSet::start(self.divide_pipeline.layout().descriptor_set_layout(0).unwrap().clone())
                .add_buffer(ctx.buffers.workgroups.clone())
                .unwrap()
                .build()
                .unwrap()
        );

        let mut command = ctx
            .create_command_buffer()
            .dispatch(1, self.divide_pipeline.clone(), set)
            .unwrap()
            .build();

        commands.push(command);
    }
}
