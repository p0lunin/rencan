use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::{
        descriptor_set::PersistentDescriptorSet, pipeline_layout::PipelineLayout,
        PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

use crate::{
    commands::shaders::ray_trace_shader,
    core::{
        intersection::IntersectionUniform, BufferAccessData, CommandFactory, CommandFactoryContext,
        Ray,
    },
};
use vulkano::buffer::{DeviceLocalBuffer, BufferUsage, TypedBufferAccess};
use crate::core::app::GlobalAppBuffers;

mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}

mod make_shadow_rays_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/make_shadow_rays.glsl"
    }
}

pub struct LightningCommandFactory {
    device: Arc<Device>,
    ray_trace_pipeline: Arc<ComputePipeline<PipelineLayout<ray_trace_shader::Layout>>>,
    make_shadow_rays_pipeline: Arc<ComputePipeline<PipelineLayout<make_shadow_rays_cs::Layout>>>,
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
    global_buffers: GlobalAppBuffers,
    local_ray_buffer: Arc<DeviceLocalBuffer<[Ray]>>,
    local_intersections_buffer: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
}

impl LightningCommandFactory {
    pub fn new(buffers: GlobalAppBuffers, device: Arc<Device>) -> Self {
        let ray_trace_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &ray_trace_shader::Shader::load(device.clone()).unwrap().main_entry_point(),
                &(),
                None,
            )
            .unwrap(),
        );
        let make_shadow_rays_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &make_shadow_rays_cs::Shader::load(device.clone()).unwrap().main_entry_point(),
                &(),
                None,
            )
            .unwrap(),
        );
        let lightning_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &lightning_cs::Shader::load(device.clone()).unwrap().main_entry_point(),
                &(),
                None,
            )
            .unwrap(),
        );
        let local_ray_buffer = DeviceLocalBuffer::array(
            device.clone(),
            buffers.rays.len(),
            BufferUsage {
                storage_buffer: true,
                transfer_destination: true,
                ..BufferUsage::none()
            },
            buffers.rays.queue_families()
        ).unwrap();
        let local_intersections_buffer = DeviceLocalBuffer::array(
            device.clone(),
            buffers.intersections.len(),
            BufferUsage {
                storage_buffer: true,
                transfer_destination: true,
                ..BufferUsage::none()
            },
            buffers.intersections.queue_families()
        ).unwrap();
        LightningCommandFactory {
            device,
            ray_trace_pipeline,
            make_shadow_rays_pipeline,
            lightning_pipeline,
            global_buffers: buffers,
            local_ray_buffer,
            local_intersections_buffer
        }
    }
}

impl CommandFactory for LightningCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer {
        let mut command = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();

        command.fill_buffer(self.local_intersections_buffer.clone(), 0).unwrap();
        command.fill_buffer(self.local_ray_buffer.clone(), 0).unwrap();

        let rays = add_making_shadow_rays(self, &self.global_buffers, &ctx, self.local_ray_buffer.clone(), &mut command);
        let intersections = add_ray_tracing(self, &ctx, rays.clone(), self.local_intersections_buffer.clone(), &mut command);
        add_lightning(self, &self.global_buffers, &ctx, rays, intersections, &mut command);

        let command = command.build().unwrap();

        command
    }

    fn update_buffers(&mut self, buffers: GlobalAppBuffers) {
        self.local_ray_buffer = DeviceLocalBuffer::array(
            self.device.clone(),
            buffers.rays.len(),
            BufferUsage::all(),
            buffers.rays.queue_families()
        ).unwrap();
        self.local_intersections_buffer = DeviceLocalBuffer::array(
            self.device.clone(),
            buffers.intersections.len(),
            BufferUsage::all(),
            buffers.intersections.queue_families()
        ).unwrap();
        self.global_buffers = buffers;
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    global_buffers: &GlobalAppBuffers,
    ctx: &CommandFactoryContext,
    rays: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    intersections: Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync>,
    command: &mut AutoCommandBufferBuilder,
) {
    let CommandFactoryContext { app_info, buffers, count_of_workgroups, scene } = ctx;
    let device = app_info.device.clone();

    let layout_0 = factory.lightning_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(global_buffers.rays.clone())
            .unwrap()
            .add_buffer(rays.clone())
            .unwrap()
            .add_image(buffers.output_image.clone())
            .unwrap()
            .add_buffer(global_buffers.intersections.clone())
            .unwrap()
            .add_buffer(intersections.clone())
            .unwrap()
            .add_buffer(buffers.direction_light.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let layout_1 = factory.lightning_pipeline.layout().descriptor_set_layout(1).unwrap();

    for (i, model) in scene.models.iter().enumerate() {
        let vertices_buffer = model.get_vertices_buffer(&app_info.graphics_queue);
        let model_info_buffer = model.get_info_buffer(&device, i as u32);
        let indices_buffer = model.get_indices_buffer(&app_info.graphics_queue);

        let set_1 = Arc::new(
            PersistentDescriptorSet::start(layout_1.clone())
                .add_buffer(model_info_buffer)
                .unwrap()
                .add_buffer(vertices_buffer)
                .unwrap()
                .add_buffer(indices_buffer)
                .unwrap()
                .build()
                .unwrap(),
        );

        command
            .dispatch(
                [*count_of_workgroups, 1, 1],
                factory.lightning_pipeline.clone(),
                (set_0.clone(), set_1),
                (),
            )
            .unwrap();
    }
}

fn add_making_shadow_rays(
    factory: &LightningCommandFactory,
    global_buffers: &GlobalAppBuffers,
    ctx: &CommandFactoryContext,
    new_rays_buffer: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    command: &mut AutoCommandBufferBuilder,
) -> Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync> {
    let CommandFactoryContext { buffers, count_of_workgroups, .. } = ctx;

    let layout_0 = factory.make_shadow_rays_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(global_buffers.rays.clone())
            .unwrap()
            .add_buffer(global_buffers.intersections.clone())
            .unwrap()
            .add_buffer(new_rays_buffer.clone())
            .unwrap()
            .add_buffer(buffers.direction_light.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    command
        .dispatch(
            [*count_of_workgroups, 1, 1],
            factory.make_shadow_rays_pipeline.clone(),
            set_0,
            (),
        )
        .unwrap();

    new_rays_buffer
}

fn add_ray_tracing(
    factory: &LightningCommandFactory,
    ctx: &CommandFactoryContext,
    rays: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    command: &mut AutoCommandBufferBuilder,
) -> Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync> {
    let CommandFactoryContext { app_info, buffers, count_of_workgroups, scene } = ctx;
    let device = app_info.device.clone();

    let layout_0 = factory.ray_trace_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(rays.clone())
            .unwrap()
            .add_buffer(intersections.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let layout_1 = factory.ray_trace_pipeline.layout().descriptor_set_layout(1).unwrap();

    for (i, model) in scene.models.iter().enumerate() {
        let vertices_buffer = model.get_vertices_buffer(&app_info.graphics_queue);
        let model_info_buffer = model.get_info_buffer(&device, i as u32);
        let indices_buffer = model.get_indices_buffer(&app_info.graphics_queue);

        let set_1 = Arc::new(
            PersistentDescriptorSet::start(layout_1.clone())
                .add_buffer(model_info_buffer)
                .unwrap()
                .add_buffer(vertices_buffer)
                .unwrap()
                .add_buffer(indices_buffer)
                .unwrap()
                .build()
                .unwrap(),
        );

        command
            .dispatch(
                [*count_of_workgroups, 1, 1],
                factory.ray_trace_pipeline.clone(),
                (set_0.clone(), set_1),
                (),
            )
            .unwrap();
    }

    intersections
}
