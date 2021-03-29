use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::pipeline_layout::PipelineLayout,
    device::Device,
    pipeline::ComputePipeline,
};

use crate::core::{CommandFactory, CommandFactoryContext, AutoCommandBufferBuilderWrap};
use vulkano::command_buffer::{CommandBuffer, Kind};
use vulkano::command_buffer::sys::{UnsafeCommandBuffer, UnsafeCommandBufferBuilder, UnsafeCommandBufferBuilderPipelineBarrier, Flags};
use vulkano::framebuffer::{FramebufferAbstract, EmptySinglePassRenderPassDesc, RenderPass};
use vulkano::command_buffer::pool::CommandPool;
use vulkano::sync::{PipelineStages, AccessFlagBits};
use vulkano::image::{ImageLayout, ImageAccess};

pub mod blue_sky_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/sky/blue.glsl",
        include: ["shaders"]
    }
}

pub struct SkyCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<blue_sky_cs::Layout>>>,
    local_size_x: u32,
}

impl SkyCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let local_size_x = device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = blue_sky_cs::SpecializationConstants {
            constant_0: local_size_x,
        };

        let pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &blue_sky_cs::Shader::load(device).unwrap().main_entry_point(),
                &constants,
                None,
            )
            .unwrap(),
        );
        SkyCommandFactory { pipeline, local_size_x }
    }
}

impl CommandFactory for SkyCommandFactory {
    fn make_command(&mut self, ctx: CommandFactoryContext, commands: &mut Vec<Box<dyn CommandBuffer>>) {
        let set_0 = ctx.buffers.global_app_set.clone();
        let set_1 = ctx.buffers.image_set.clone();

        let command = ctx
            .create_command_buffer()
            .dispatch(ctx.app_info.size_of_image_array() as u32 / self.local_size_x, self.pipeline.clone(), (set_0, set_1))
            .unwrap()
            .build();

        commands.push(command);
    }
}

unsafe fn make_barrier(ctx: &CommandFactoryContext) -> UnsafeCommandBuffer {
    use vulkano::command_buffer::pool::CommandPoolBuilderAlloc;
    use vulkano::command_buffer::pool::CommandPoolAlloc;
    let pool = Device::standard_command_pool(
        &ctx.app_info.device.clone(),
        ctx.app_info.graphics_queue.family(),
    );
    let pool_builder_alloc = pool
        .alloc(false, 1).unwrap()
        .next().unwrap();
    let mut buffer = UnsafeCommandBufferBuilder::new::<
        RenderPass<EmptySinglePassRenderPassDesc>,
        Box<dyn FramebufferAbstract>
    >(
        pool_builder_alloc.inner(),
        Kind::Primary,
        Flags::None,
    ).unwrap();

    let mut builder = UnsafeCommandBufferBuilderPipelineBarrier::new();
    builder.add_image_memory_barrier(
        &ctx.buffers.image.image(),
        0..1,
        0..1,
        PipelineStages {
            compute_shader: true,
            ..PipelineStages::none()
        },
        AccessFlagBits {
            memory_write: true,
            shader_write: true,
            ..AccessFlagBits::none()
        },
        PipelineStages {
            compute_shader: true,
            ..PipelineStages::none()
        },
        AccessFlagBits {
            memory_write: true,
            shader_write: true,
            ..AccessFlagBits::none()
        },
        false,
        None,
        ctx.buffers.image.image().final_layout_requirement(),
        ctx.buffers.image.image().final_layout_requirement()
    );


    buffer.pipeline_barrier(
        &builder
    );

    buffer.build().unwrap()
}
