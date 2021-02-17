use std::sync::Arc;

use crevice::std140::AsStd140;
use nalgebra::Point3;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::{descriptor_set::PersistentDescriptorSet, PipelineLayoutAbstract},
    image::ImageViewAccess,
    pipeline::ComputePipeline,
};

use rencan_core::{AppInfo, BufferAccessData, Screen};

use rencan_core::app::Rays;

#[allow(dead_code)]
pub fn show_xyz_ordinates(
    AppInfo { screen, device, graphics_queue: queue, .. }: &AppInfo,
    origin: Point3<f32>,
    image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    rays: Arc<dyn BufferAccessData<Data = Rays> + Send + Sync + 'static>,
    screen_buffer: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync + 'static>,
) -> AutoCommandBuffer {
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "shaders/show_xyz_ordinates.glsl"
        }
    }

    let shader = cs::Shader::load(device.clone()).unwrap();

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None).unwrap(),
    );

    let origin_buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::all(),
        false,
        Into::<mint::Vector3<f32>>::into(origin.coords).as_std140(),
    )
    .unwrap();

    let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set = Arc::new(
        PersistentDescriptorSet::start(layout.clone())
            .add_buffer(screen_buffer.clone())
            .unwrap()
            .add_buffer(origin_buffer)
            .unwrap()
            .add_buffer(rays)
            .unwrap()
            .add_image(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let mut command = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    command
        .dispatch([screen.width() * screen.height(), 1, 1], compute_pipeline.clone(), set, ())
        .unwrap();
    let command = command.build().unwrap();

    command
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

        let app_info =
            AppInfo::new(instance.clone(), queue.clone(), device.clone(), screen.clone());

        let command = show_xyz_ordinates(
            &app_info,
            camera.position().clone(),
            image.clone(),
            rays_buffer.clone(),
            to_buffer(device.clone(), screen.clone()),
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

        assert_eq!(pixels_refs.as_slice(), expected.as_slice(),);
    }
}
*/
