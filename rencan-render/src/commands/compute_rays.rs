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

use crate::core::CommandFactoryContext;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/compute_rays.glsl"
    }
}

pub struct ComputeRaysCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
}

impl ComputeRaysCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None).unwrap(),
        );
        ComputeRaysCommandFactory { pipeline }
    }
}

impl CommandFactory for ComputeRaysCommandFactory {
    fn make_command(
        &self,
        ctx: CommandFactoryContext,
    ) -> AutoCommandBuffer<StandardCommandPoolAlloc> {
        let mut calc_rays = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();
        calc_rays
            .dispatch([ctx.count_of_workgroups, 1, 1], self.pipeline.clone(), ctx.global_set, ())
            .unwrap();
        let calc_rays_command = calc_rays.build().unwrap();

        calc_rays_command
    }
}

#[cfg(test)]
mod tests {
    use crevice::std140::AsStd140;
    use nalgebra::{Point3, Rotation3};

    use rencan_core::camera::Camera;

    use crate::test_utils::*;

    use super::*;

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
                9,
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
                9,
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
