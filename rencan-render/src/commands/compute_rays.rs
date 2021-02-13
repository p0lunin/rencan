use crate::{app::Rays, camera::CameraUniform};
use crevice::std140::AsStd140;
use rencan_core::{AppInfo, BufferAccessData, Screen};
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::{descriptor_set::PersistentDescriptorSet, PipelineLayoutAbstract},
    pipeline::ComputePipeline,
};

pub fn compute_rays(
    AppInfo { graphics_queue: queue, screen, device, .. }: &AppInfo,
    screen_buffer: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync + 'static>,
    camera_buffer: Arc<
        dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type>
            + Send
            + Sync
            + 'static,
    >,
    rays_out_buffer: Arc<dyn BufferAccessData<Data = Rays> + Send + Sync + 'static>,
) -> AutoCommandBuffer {
    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "shaders/compute_rays.glsl"
        }
    }

    let shader = cs::Shader::load(device.clone()).unwrap();

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None).unwrap(),
    );

    let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set = Arc::new(
        PersistentDescriptorSet::start(layout.clone())
            .add_buffer(screen_buffer)
            .unwrap()
            .add_buffer(camera_buffer)
            .unwrap()
            .add_buffer(rays_out_buffer)
            .unwrap()
            .build()
            .unwrap(),
    );

    let mut calc_rays = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    calc_rays
        .dispatch([screen.width() * screen.height(), 1, 1], compute_pipeline.clone(), set, ())
        .unwrap();
    let calc_rays_command = calc_rays.build().unwrap();

    calc_rays_command
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{camera::Camera, test_utils::*};
    use crevice::std140::AsStd140;
    use nalgebra::{Point3, Rotation3};

    #[test]
    fn test_compute() {
        let instance = init_vk_instance();
        let (device, queue) = pick_device_and_queue(&instance);
        let screen = Screen::new(3, 3);
        let camera = Camera::from_origin();
        let rays_buffer = empty_rays(device.clone(), 3 * 3);

        let app_info =
            AppInfo::new(instance.clone(), queue.clone(), device.clone(), screen.clone());

        run_one(
            compute_rays(
                &app_info,
                to_buffer(device.clone(), screen.clone()),
                to_buffer(device.clone(), camera.into_uniform().as_std140()),
                rays_buffer.clone(),
            ),
            queue.clone(),
        );
        let rays: Vec<[f32; 3]> = rays_buffer
            .read()
            .unwrap()
            .iter()
            .cloned()
            .map(|p| [p.x, p.y, p.z])
            .collect::<Vec<_>>();
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

        approx::assert_abs_diff_eq!(rays_refs.as_slice(), expected.as_slice(), epsilon = 0.0001);
    }

    #[test]
    fn test_rotate_camera() {
        let instance = init_vk_instance();
        let (device, queue) = pick_device_and_queue(&instance);
        let screen = Screen::new(3, 3);
        let camera = Camera::new(
            Point3::new(0.0, 0.0, 0.0),
            Rotation3::from_euler_angles(0.0, -std::f32::consts::FRAC_PI_2, 0.0),
        );
        let rays_buffer = empty_rays(device.clone(), 3 * 3);

        let app_info =
            AppInfo::new(instance.clone(), queue.clone(), device.clone(), screen.clone());

        run_one(
            compute_rays(
                &app_info,
                to_buffer(device.clone(), screen.clone()),
                to_buffer(device.clone(), camera.into_uniform().as_std140()),
                rays_buffer.clone(),
            ),
            queue.clone(),
        );
        let rays: Vec<[f32; 3]> = rays_buffer
            .read()
            .unwrap()
            .iter()
            .cloned()
            .map(|p| [p.x, p.y, p.z])
            .collect::<Vec<_>>();
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

        approx::assert_abs_diff_eq!(rays_refs.as_slice(), expected.as_slice(), epsilon = 0.0001);
    }
}
