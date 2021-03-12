use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::pipeline_layout::PipelineLayout,
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/ray_tracing.glsl"
    }
}

pub mod ray_trace_shader {
    pub use super::cs::*;
}

pub struct RayTraceCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl RayTraceCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None).unwrap(),
        );
        RayTraceCommandFactory { pipeline }
    }
}

impl CommandFactory for RayTraceCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer {
        let CommandFactoryContext { app_info, buffers, count_of_workgroups, .. } = ctx;
        let device = app_info.device.clone();

        let set_0 = buffers.global_app_set.clone();

        let set_1 = buffers.models_set.clone();

        let mut command =
            AutoCommandBufferBuilder::new(device.clone(), app_info.graphics_queue.family())
                .unwrap();

        command
            .dispatch([count_of_workgroups, 1, 1], self.pipeline.clone(), (set_0, set_1), ())
            .unwrap();

        let command = command.build().unwrap();

        command
    }
}
/*
#[cfg(test)]
mod tests {
    use nalgebra::{Point3, Point4, Rotation3};
    use vulkano::{
        command_buffer::CommandBuffer,
        image::{AttachmentImage, ImageUsage},
        sync::GpuFuture,
    };

    use rencan_core::camera::Camera;

    use crate::test_utils::*;

    use super::*;

    #[test]
    fn test_show() {
        let instance = init_vk_instance();
        let (device, queue) = pick_device_and_queue(&instance);
        let screen = Screen::new(3, 3);
        let camera = Camera::new(Point3::new(0.0, 0.0, 1.0), Rotation3::identity());
        let image = AttachmentImage::with_usage(
            device.clone(),
            screen.0.clone(),
            vulkano::format::R8G8B8A8Uint,
            ImageUsage { storage: true, transfer_source: true, ..ImageUsage::none() },
        )
        .unwrap();

        let rays: Vec<Point4<f32>> = vec![
            [-1.0, 1.0, -1.0, 0.0].into(),
            [0.0, 1.0, -1.0, 0.0].into(),
            [1.0, 1.0, -1.0, 0.0].into(),
            [-1.0, 0.0, -1.0, 0.0].into(),
            [0.0, 0.0, -1.0, 0.0].into(),
            [1.0, 0.0, -1.0, 0.0].into(),
            [-1.0, -1.0, -1.0, 0.0].into(),
            [0.0, -1.0, -1.0, 0.0].into(),
            [1.0, -1.0, -1.0, 0.0].into(),
        ];

        let rays_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            rays.into_iter(),
        )
        .unwrap();

        let app_info =
            AppInfo::new(instance.clone(), queue.clone(), device.clone(), screen.clone());

        let command = make_ray_tracing(
            &app_info,
            9,
            std::iter::once(&Model::new(vec![], vec![])),
            camera.position().clone(),
            image.clone(),
            to_buffer(device.clone(), screen.clone()),
            rays_buffer,
        );
        let image_buf = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            (0..3 * 3).map(|_| [0; 4]),
        )
        .unwrap();

        let copy_command =
            copy_image_to_buf(device.clone(), queue.clone(), image.clone(), image_buf.clone());

        command
            .execute(queue)
            .unwrap()
            .then_execute_same_queue(copy_command)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let pixels: Vec<[u8; 4]> = image_buf.read().unwrap().iter().cloned().collect::<Vec<_>>();
        let pixels_refs: Vec<&[u8]> = pixels.iter().map(|x| x as &[u8]).collect();

        let expected: Vec<&[u8]> = vec![
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
        ];

        assert_eq!(pixels_refs.as_slice(), expected.as_slice(),);
    }
}
*/
