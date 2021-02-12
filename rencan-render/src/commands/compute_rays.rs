use vulkano::{
    device::{Device, Queue},
    buffer::{CpuAccessibleBuffer, BufferAccess},
    command_buffer::AutoCommandBuffer,
    pipeline::ComputePipeline,
    descriptor::{PipelineLayoutAbstract, descriptor_set::PersistentDescriptorSet},
};
use std::sync::Arc;
use nalgebra::Point4;
use rencan_core::Screen;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use crate::camera::Camera;

pub fn compute_rays<Cam>(
    screen: Screen,
    device: Arc<Device>,
    queue: Arc<Queue>,
    screen_buffer: Arc<CpuAccessibleBuffer<Screen>>,
    camera_buffer: Arc<CpuAccessibleBuffer<Cam>>,
    rays_out_buffer: Arc<CpuAccessibleBuffer<[Point4<f32>]>>,
) -> AutoCommandBuffer
where
    Arc<CpuAccessibleBuffer<Cam>>: BufferAccess,
    Cam: Send + Sync + 'static,
{
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "shaders/compute_rays.glsl"
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

    let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set = Arc::new(
        PersistentDescriptorSet::start(layout.clone())
            .add_buffer(screen_buffer).unwrap()
            .add_buffer(camera_buffer).unwrap()
            .add_buffer(rays_out_buffer).unwrap()
            .build().unwrap()
    );

    let mut calc_rays = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    calc_rays.dispatch([screen.width()*screen.height(), 1, 1], compute_pipeline.clone(), set, ()).unwrap();
    let calc_rays_command = calc_rays.build().unwrap();

    calc_rays_command
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crevice::std140::AsStd140;
    use nalgebra::{Point3, Rotation3};

    #[test]
    fn test_compute() {
        let instance = init_vk_instance();
        let (device, queue) = pick_device_and_queue(&instance);
        let screen = Screen::new(3, 3);
        let camera = Camera::from_origin();
        let rays_buffer = empty_rays(device.clone(), 3*3);
        run_one(compute_rays(
            screen.clone(),
            device.clone(),
            queue.clone(),
            to_buffer(device.clone(), screen.clone()),
            to_buffer(device.clone(), camera.into_uniform().as_std140()),
            rays_buffer.clone()
        ),
        queue.clone()
        );
        let rays: Vec<[f32; 3]> = rays_buffer.read().unwrap().iter().cloned().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>();
        let rays_refs: Vec<&[f32]> = rays.iter().map(|x| x as &[f32]).collect();

        let expected: Vec<&[f32]> = vec![
            &[-1.0, 1.0, -1.0],
            &[0.0, 1.0, -1.0],
            &[1.0, 1.0, -1.0],

            &[-1.0, 0.0, -1.0],
            &[0.0, 0.0, -1.0],
            &[1.0, 0.0, -1.0],

            &[-1.0, -1.0, -1.0],
            &[0.0, -1.0, -1.0],
            &[1.0, -1.0, -1.0],
        ];

        approx::assert_abs_diff_eq!(
            rays_refs.as_slice(),
            expected.as_slice(),
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_rotate_camera() {
        let instance = init_vk_instance();
        let (device, queue) = pick_device_and_queue(&instance);
        let screen = Screen::new(3, 3);
        let camera = Camera::new(
            Point3::new(0.0, 0.0, 0.0),
            Rotation3::from_euler_angles(
                0.0,
                -std::f32::consts::FRAC_PI_2,
                0.0,
            )
        );
        let rays_buffer = empty_rays(device.clone(), 3*3);
        run_one(compute_rays(
            screen.clone(),
            device.clone(),
            queue.clone(),
            to_buffer(device.clone(), screen.clone()),
            to_buffer(device.clone(), camera.into_uniform().as_std140()),
            rays_buffer.clone()
        ),
        queue.clone()
        );
        let rays: Vec<[f32; 3]> = rays_buffer.read().unwrap().iter().cloned().map(|p| [p.x, p.y, p.z]).collect::<Vec<_>>();
        let rays_refs: Vec<&[f32]> = rays.iter().map(|x| x as &[f32]).collect();

        let expected: Vec<&[f32]> = vec![
            &[1.0, 1.0, -1.0],
            &[1.0, 1.0, 0.0],
            &[1.0, 1.0, 1.0],

            &[1.0, 0.0, -1.0],
            &[1.0, 0.0, 0.0],
            &[1.0, 0.0, 1.0],

            &[1.0, -1.0, -1.0],
            &[1.0, -1.0, 0.0],
            &[1.0, -1.0, 1.0],
        ];

        approx::assert_abs_diff_eq!(
            rays_refs.as_slice(),
            expected.as_slice(),
            epsilon = 0.0001
        );
    }
}