use rencan_core::Screen;
use nalgebra::{Point3, Point4};
use std::sync::Arc;
use vulkano::{
    device::{Device, Queue},
    buffer::{CpuAccessibleBuffer},
    command_buffer::AutoCommandBuffer,
    pipeline::ComputePipeline,
    descriptor::{PipelineLayoutAbstract, descriptor_set::PersistentDescriptorSet},
};
use vulkano::image::{ImageViewAccess};
use vulkano::buffer::BufferUsage;
use crevice::std140::AsStd140;
use vulkano::command_buffer::AutoCommandBufferBuilder;

fn show_xyz_ordinates(
    screen: Screen,
    origin: Point3<f32>,
    device: Arc<Device>,
    image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    queue: Arc<Queue>,
    rays: Arc<CpuAccessibleBuffer<[Point4<f32>]>>,
    screen_buffer: Arc<CpuAccessibleBuffer<Screen>>,
) -> AutoCommandBuffer {

    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "shaders/show_xyz_ordinates.glsl"
        }
    }

    let shader = cs::Shader::load(device.clone()).unwrap();

    let compute_pipeline = Arc::new(
        ComputePipeline::new(
            device.clone(),
            &shader.main_entry_point(),
            &(),
            None
        ).unwrap()
    );

    let origin_buffer =
        CpuAccessibleBuffer::from_data(
            device.clone(),
            BufferUsage::all(),
            false,
            Into::<mint::Vector3<f32>>::into(origin.coords).as_std140()
        ).unwrap();

    let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set = Arc::new(
        PersistentDescriptorSet::start(layout.clone())
            .add_buffer(screen_buffer.clone()).unwrap()
            .add_buffer(origin_buffer).unwrap()
            .add_buffer(rays).unwrap()
            .add_image(image.clone()).unwrap()
            .build().unwrap()
    );

    let mut command = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    command.dispatch([screen.width() * screen.height(), 1, 1], compute_pipeline.clone(), set, ()).unwrap();
    let command = command.build().unwrap();

    command
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use nalgebra::{Point3, Rotation3};
    use crate::camera::Camera;
    use vulkano::image::{AttachmentImage, ImageUsage};
    use vulkano::command_buffer::CommandBuffer;
    use vulkano::sync::GpuFuture;

    #[test]
    fn test_show() {
        let instance = init_vk_instance();
        let (device, queue) = pick_device_and_queue(&instance);
        let screen = Screen::new(3, 3);
        let camera = Camera::new(
            Point3::new(0.0, 0.0, 1.0),
            Rotation3::identity(),
        );
        let image = AttachmentImage::with_usage(
            device.clone(),
            screen.0.clone(),
            vulkano::format::R8G8B8A8Uint,
            ImageUsage {
                storage: true,
                transfer_source: true,
                ..ImageUsage::none()
            }
        ).unwrap();
        let rays = vec![
            Point4::new(-1.0, 1.0, -1.0, 0.0),
            Point4::new(0.0, 1.0, -1.0, 0.0),
            Point4::new(1.0, 1.0, -1.0, 0.0),

            Point4::new(-1.0, 0.0, -1.0, 0.0),
            Point4::new(0.0, 0.0, -1.0, 0.0),
            Point4::new(1.0, 0.0, -1.0, 0.0),

            Point4::new(-1.0, -1.0, -1.0, 0.0),
            Point4::new(0.0, -1.0, -1.0, 0.0),
            Point4::new(1.0, -1.0, -1.0, 0.0),
        ];
        let rays_buffer = rays_from_vec(device.clone(), rays);
        let command = show_xyz_ordinates(
            screen.clone(),
            camera.position().clone(),
            device.clone(),
            image.clone(),
            queue.clone(),
            rays_buffer.clone(),
            to_buffer(device.clone(), screen.clone()),
        );
        let image_buf = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            (0..3*3).map(|_| [0; 4])
        ).unwrap();

        let copy_command = copy_image_to_buf(
            device.clone(),
            queue.clone(),
            image.clone(),
            image_buf.clone()
        );

        command.execute(queue)
            .unwrap()
            .then_execute_same_queue(copy_command).unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let pixels: Vec<[u8; 4]> = image_buf.read().unwrap().iter().cloned().collect::<Vec<_>>();
        let pixels_refs: Vec<&[u8]> = pixels.iter().map(|x| x as &[u8]).collect();

        let expected: Vec<&[u8]> = vec![
            &[0, 0, 0, 255],
            &[0, 255, 0, 255],
            &[0, 0, 0, 255],

            &[0, 0, 0, 255],
            &[255, 255, 255, 255],
            &[255, 0, 0, 255],

            &[0, 0, 0, 255],
            &[0, 0, 0, 255],
            &[0, 0, 0, 255],
        ];

        assert_eq!(
            pixels_refs.as_slice(),
            expected.as_slice(),
        );
    }
}
