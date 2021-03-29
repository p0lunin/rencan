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

use crate::core::{camera::Camera, CommandFactoryContext, Screen};
use nalgebra::Point3;
use std::cell::RefCell;
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::PipelineLayoutAbstract;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/squeeze.glsl"
    }
}

pub struct SqueezeCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::Layout>>>,
    local_image: Option<Arc<ImageView<AttachmentImage>>>,
    prev_screen: Option<Screen>,
    local_size_x: u32,
}

impl SqueezeCommandFactory {
    pub fn new(device: Arc<Device>) -> Self {
        let shader = cs::Shader::load(device.clone()).unwrap();
        let local_size_x = device.physical_device().extended_properties().subgroup_size().unwrap_or(32);

        let constants = cs::SpecializationConstants {
            constant_0: local_size_x,
        };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None).unwrap(),
        );
        SqueezeCommandFactory {
            pipeline,
            local_image: None,
            prev_screen: None,
            local_size_x
        }
    }

    fn init_image(&mut self, ctx: &CommandFactoryContext) -> Arc<AttachmentImage> {
        let image = AttachmentImage::with_usage(
            ctx.app_info.device.clone(),
            ctx.app_info.screen.0,
            ctx.buffers.image.format(),
            ImageUsage {
                storage: true,
                transfer_source: true,
                ..ImageUsage::none()
            },
        ).unwrap();
        self.local_image = Some(image.clone());
        image
    }
}

impl CommandFactory for SqueezeCommandFactory {
    fn make_command(
        &mut self,
        ctx: CommandFactoryContext,
        commands: &mut Vec<AutoCommandBuffer>,
    ) {
        let local_image = match &self.local_image {
            Some(i) if *self.prev_screen.as_ref().unwrap() == ctx.app_info.screen => i.clone(),
            _ => {
                self.prev_screen = Some(ctx.app_info.screen.clone());
                self.init_image(&ctx)
            }
        };

        let mut command = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();

        let set_0 = ctx.buffers.global_app_set.clone();
        let set_1 = ctx.buffers.image_set.clone();
        let set_2 = Arc::new(PersistentDescriptorSet::start(
            self.pipeline.layout().descriptor_set_layout(2).unwrap().clone()
        )
            .add_image(local_image.clone())
            .unwrap()
            .build().unwrap()
        );

        let sets = (set_0, set_1, set_2);

        command
            .dispatch([ctx.app_info.size_of_image_array() as u32 / self.local_size_x, 1, 1], self.pipeline.clone(), sets, ())
            .unwrap();

        let command = command.build().unwrap();

        let mut copy_command = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();
        copy_command.copy_image(
            local_image.clone(),
            [0, 0, 0],
            0,
            0,
            ctx.buffers.image.clone(),
            [0,0,0],
            0,
            0,
            local_image.dimensions().width_height_depth(),
            1
        ).unwrap();

        let copy_command = copy_command.build().unwrap();

        commands.push(command);
        commands.push(copy_command)
    }
}
