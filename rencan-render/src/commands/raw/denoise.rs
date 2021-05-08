use crate::core::{CommandFactoryContext, AppInfo};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferAccess, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, DispatchIndirectCommand},
    descriptor::{
        descriptor_set::UnsafeDescriptorSetLayout,
        pipeline_layout::{PipelineLayout},
        DescriptorSet, PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::format::Format;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/denoise.glsl"
    }
}

pub struct DenoiseCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::MainLayout>>>,
    output_image: Arc<ImageView<Arc<AttachmentImage>>>,
}

impl DenoiseCommandFactory {
    pub fn new(info: &AppInfo, format: Format, dims: [u32; 2]) -> Self {
        let device = &info.device;
        let shader = cs::Shader::load(device.clone()).unwrap();
        let local_size_x = info.recommend_workgroups_length;

        let constants = cs::SpecializationConstants { constant_0: local_size_x };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        let output_image = ImageView::new(
            AttachmentImage::with_usage(
                device.clone(),
                dims,
                format,
                ImageUsage {
                    storage: true,
                    transfer_destination: true,
                    transfer_source: true,
                    ..ImageUsage::none()
                }
            ).unwrap()
        ).unwrap();
        DenoiseCommandFactory { pipeline, output_image }
    }

    pub fn add_denoise<IIS>(
        &self,
        ctx: &CommandFactoryContext,
        input_image_set: IIS,
        buffer: &mut AutoCommandBufferBuilder,
    ) -> Arc<ImageView<Arc<AttachmentImage>>>
        where
        IIS: DescriptorSet + Send + Sync + 'static,
    {
        let output_image_set = PersistentDescriptorSet::start(self.output_image_layout())
            .add_image(self.output_image.clone())
            .unwrap()
            .build()
            .unwrap();

        let sets = (
            ctx.buffers.global_app_set.clone(),
            input_image_set,
            output_image_set
        );

        buffer
            .dispatch(
                [ctx.app_info.screen.size(), 1, 1],
                self.pipeline.clone(),
                sets,
                (),
                std::iter::empty(),
            )
            .unwrap();

        self.output_image.clone()
    }

    pub fn input_image_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(1).unwrap().clone()
    }

    pub fn output_image_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(2).unwrap().clone()
    }
}
