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
use std::io::Cursor;
use vulkano::image::{ImageDimensions, AttachmentImage, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::command_buffer::CommandBuffer;
use vulkano::format::Format;
use vulkano::sync::GpuFuture;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning/make_gi_rays.glsl"
    }
}

pub struct MakeGiRaysCommandFactory {
    pipeline: Arc<ComputePipeline<PipelineLayout<cs::MainLayout>>>,
    random_values: Vec<f32>,
    noise_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl MakeGiRaysCommandFactory {
    pub fn new(info: &AppInfo) -> Self {
        let device = &info.device;
        let shader = cs::Shader::load(device.clone()).unwrap();
        let local_size_x = info.recommend_workgroups_length;

        let shader = cs::Shader::load(device.clone()).unwrap();
        let constants = cs::SpecializationConstants { constant_0: local_size_x, };
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &constants, None)
                .unwrap(),
        );
        let random_values = (0..2048).into_iter().map(|_| rand::random()).collect();

        let (texture, tex_future) = {
            let png_bytes = include_bytes!("../../../img/blue_noise.png").to_vec();
            let cursor = Cursor::new(png_bytes);
            let decoder = png::Decoder::new(cursor);
            let (img_info, mut reader) = decoder.read_info().unwrap();
            let dimensions = ImageDimensions::Dim2d {
                width: img_info.width,
                height: img_info.height,
                array_layers: 1,
            };
            let mut image_data = Vec::new();
            image_data.resize((img_info.width * img_info.height * 4) as usize, 0);
            reader.next_frame(&mut image_data).unwrap();

            let image = AttachmentImage::with_usage(
                device.clone(),
                [img_info.width, img_info.height],
                Format::R8G8B8A8Unorm,
                ImageUsage {
                    storage: true,
                    transfer_destination: true,
                    ..ImageUsage::none()
                }
            )
            .unwrap();

            let buf = CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage {
                    storage_buffer: true,
                    transfer_source: true,
                    ..BufferUsage::none()
                },
                false,
                image_data.into_iter(),
            ).unwrap();

            let mut cmd = AutoCommandBufferBuilder::new(
                device.clone(),
                info.graphics_queue.family()
            ).unwrap();

            cmd.copy_buffer_to_image(buf, image.clone()).unwrap();

            let future = cmd
                .build()
                .unwrap()
                .execute(info.graphics_queue.clone())
                .unwrap();

            (ImageView::new(image).unwrap(), future)
        };

        tex_future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

        let noise_set = PersistentDescriptorSet::start(
            pipeline.layout().descriptor_set_layout(7).unwrap().clone()
        )
            .add_image(texture)
            .unwrap()
            .build()
            .unwrap();

        let noise_set = Arc::new(noise_set);

        MakeGiRaysCommandFactory { pipeline, random_values, noise_set }
    }

    pub fn add_making_gi_rays<PIS, WI, WOS, IntersSer, GTS>(
        &self,
        sample_number: u32,
        ctx: &CommandFactoryContext,
        workgroups_input: WI,
        previous_intersections_set: PIS,
        intersections_set: IntersSer,
        workgroups_out_set: WOS,
        gi_thetas_set: GTS,
        buffer: &mut AutoCommandBufferBuilder,
    ) where
        PIS: DescriptorSet + Send + Sync + 'static,
        WI: BufferAccess
            + TypedBufferAccess<Content = [DispatchIndirectCommand]>
            + Send
            + Sync
            + 'static,
        WOS: DescriptorSet + Send + Sync + 'static,
        IntersSer: DescriptorSet + Send + Sync + 'static,
        GTS: DescriptorSet + Send + Sync + 'static,
    {
        let sets = (
            intersections_set,
            ctx.buffers.models_set.clone(),
            ctx.buffers.sphere_models_set.clone(),
            workgroups_out_set,
            previous_intersections_set,
            gi_thetas_set,
            ctx.buffers.global_app_set.clone(),
            self.noise_set.clone(),
        );

        let (rand1, rand2) = self.give_random_numbers(sample_number);

        buffer
            .dispatch_indirect(
                workgroups_input,
                self.pipeline.clone(),
                sets,
                (
                    rand1,
                    rand2,
                    ctx.app_info.size_of_image_array() as u32 * ctx.render_step,
                    ctx.app_info.msaa
                ),
                std::iter::empty(),
            )
            .unwrap();
    }

    pub fn intersections_layout(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(0).unwrap().clone()
    }

    pub fn gi_thetas_set(&self) -> Arc<UnsafeDescriptorSetLayout> {
        self.pipeline.layout().descriptor_set_layout(5).unwrap().clone()
    }

    fn give_random_numbers(&self, sample_number: u32) -> (f32, f32) {
        let idx1 = sample_number as usize % self.random_values.len();
        let idx2 = (idx1 * 2) as usize % self.random_values.len();
        (self.random_values[idx1], self.random_values[idx2])
    }
}
