use std::sync::Arc;

use vulkano::{
    command_buffer::{
        pool::standard::StandardCommandPoolAlloc, AutoCommandBuffer, AutoCommandBufferBuilder,
    },
    descriptor::pipeline_layout::PipelineLayout,
    device::Device,
    pipeline::ComputePipeline,
};

use rencan_core::CommandFactory;

use crate::core::{camera::Camera, CommandFactoryContext, Screen};
use nalgebra::Point3;
use std::cell::RefCell;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/compute_rays.glsl"
    }
}

pub struct ComputeRaysCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
    prev_camera: RefCell<Camera>,
    prev_screen: RefCell<Screen>,
}

impl ComputeRaysCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None).unwrap(),
        );
        ComputeRaysCommandFactory {
            pipeline,
            prev_camera: RefCell::new(Camera::new(
                Point3::new(f32::NAN, f32::NAN, f32::NAN),
                (f32::NAN, f32::NAN, f32::NAN),
                0.0,
            )),
            prev_screen: RefCell::new(Screen::new(0, 0)),
        }
    }
}

impl CommandFactory for ComputeRaysCommandFactory {
    fn make_command(
        &self,
        ctx: CommandFactoryContext,
        commands: &mut Vec<AutoCommandBuffer>,
    ) {
        if *self.prev_screen.borrow() == ctx.app_info.screen
            && *self.prev_camera.borrow() == *ctx.camera
        {
            return;
        }

        *self.prev_camera.borrow_mut() = ctx.camera.clone();
        *self.prev_screen.borrow_mut() = ctx.app_info.screen.clone();

        let mut calc_rays = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();

        let set_0 = ctx.buffers.global_app_set.clone();

        calc_rays
            .dispatch([ctx.app_info.size_of_image_array() as u32 / 32, 1, 1], self.pipeline.clone(), set_0, ())
            .unwrap();

        let calc_rays_command = calc_rays.build().unwrap();

        commands.push(calc_rays_command);
    }
}
