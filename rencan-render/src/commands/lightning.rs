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

pub mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}


pub struct LightningCommandFactory {
    device: Arc<Device>,
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
}

impl LightningCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
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

        add_lightning(self, &ctx, &mut command);

        let command = command.build().unwrap();

        command
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    ctx: &CommandFactoryContext,
    command: &mut AutoCommandBufferBuilder,
) {
    let CommandFactoryContext { buffers, count_of_workgroups, .. } = ctx;

    let set_0 = buffers.global_app_set.clone();
    let set_1 = buffers.models_set.clone();
    let set_2 = buffers.lights_set.clone();

    command
        .dispatch(
            [*count_of_workgroups, 1, 1],
            factory.lightning_pipeline.clone(),
            (set_0, set_1, set_2),
            (),
        )
        .unwrap();
}