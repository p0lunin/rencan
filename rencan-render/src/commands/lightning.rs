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
        app::GlobalAppBuffers, intersection::IntersectionUniform, BufferAccessData, CommandFactory,
        CommandFactoryContext, Ray,
    },
};
use std::cell::{Cell, RefCell};
use vulkano::buffer::{BufferUsage, DeviceLocalBuffer, TypedBufferAccess};

mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}


pub struct LightningCommandFactory {
    device: Arc<Device>,
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
    global_buffers: GlobalAppBuffers,
}

impl LightningCommandFactory {
    pub fn new(buffers: GlobalAppBuffers, device: Arc<Device>) -> Self {
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
            device,
            lightning_pipeline,
            global_buffers: buffers,
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

        add_lightning(self, &self.global_buffers, &ctx, &mut command);

        let command = command.build().unwrap();

        command
    }

    fn update_buffers(&mut self, buffers: GlobalAppBuffers) {
        self.global_buffers = buffers;
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    global_buffers: &GlobalAppBuffers,
    ctx: &CommandFactoryContext,
    command: &mut AutoCommandBufferBuilder,
) {
    let CommandFactoryContext { buffers, count_of_workgroups, .. } = ctx;

    let layout_0 = factory.lightning_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(global_buffers.rays.clone())
            .unwrap()
            .add_image(buffers.output_image.clone())
            .unwrap()
            .add_buffer(global_buffers.intersections.clone())
            .unwrap()
            .add_buffer(buffers.direction_light.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.point_lights_count.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.point_lights.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let layout_1 = factory.lightning_pipeline.layout().descriptor_set_layout(1).unwrap();
    let set_1 = Arc::new(
        PersistentDescriptorSet::start(layout_1.clone())
            .add_buffer(buffers.models_buffers.count.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.infos.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.vertices.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.indices.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.hit_boxes.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    command
        .dispatch(
            [*count_of_workgroups, 1, 1],
            factory.lightning_pipeline.clone(),
            (set_0, set_1),
            (),
        )
        .unwrap();
}