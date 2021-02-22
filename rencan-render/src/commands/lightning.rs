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
use vulkano::buffer::{DeviceLocalBuffer, BufferUsage};

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
    ray_trace_pipeline: Arc<ComputePipeline<PipelineLayout<ray_trace_shader::Layout>>>,
    make_shadow_rays_pipeline: Arc<ComputePipeline<PipelineLayout<make_shadow_rays_cs::Layout>>>,
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
}

impl LightningCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
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
        LightningCommandFactory {
            ray_trace_pipeline,
            make_shadow_rays_pipeline,
            lightning_pipeline,
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

        let rays = add_making_shadow_rays(self, &ctx, &mut command);
        let intersections = add_ray_tracing(self, &ctx, rays.clone(), &mut command);
        add_lightning(self, &ctx, rays, intersections, &mut command);

        let command = command.build().unwrap();

        command
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
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
            .add_buffer(buffers.rays.clone())
            .unwrap()
            .add_buffer(rays.clone())
            .unwrap()
            .add_image(buffers.output_image.clone())
            .unwrap()
            .add_buffer(buffers.intersections.clone())
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
    ctx: &CommandFactoryContext,
    command: &mut AutoCommandBufferBuilder,
) -> Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync> {
    let CommandFactoryContext { app_info, buffers, count_of_workgroups, .. } = ctx;
    let device = app_info.device.clone();

    let new_rays_buffer = DeviceLocalBuffer::array(
        device.clone(),
        app_info.size_of_image_array(),
        BufferUsage::all(),
        std::iter::once(app_info.graphics_queue.family()),
    )
    .unwrap();

    let layout_0 = factory.make_shadow_rays_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(buffers.rays.clone())
            .unwrap()
            .add_buffer(buffers.intersections.clone())
            .unwrap()
            .add_buffer(new_rays_buffer.clone())
            .unwrap()
            .add_buffer(buffers.direction_light.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    command
        .fill_buffer(new_rays_buffer.clone(), 0)
        .unwrap()
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
    command: &mut AutoCommandBufferBuilder,
) -> Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync> {
    let CommandFactoryContext { app_info, buffers, count_of_workgroups, scene } = ctx;
    let device = app_info.device.clone();

    let intersections = DeviceLocalBuffer::array(
        device.clone(),
        app_info.size_of_image_array(),
        BufferUsage::all(),
        std::iter::once(app_info.graphics_queue.family()),
    )
    .unwrap();

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

    command.fill_buffer(intersections.clone(), 0).unwrap();

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
