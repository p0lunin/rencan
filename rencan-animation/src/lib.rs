use ffmpeg::{format::Pixel, software::scaling::Context};
use image::Rgba;
use rencan_render::{
    core::{camera::Camera, AppInfo, Scene, Screen},
    App, AppBuilder, AppBuilderRtExt,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::AutoCommandBufferBuilder,
    device::{Device, DeviceExtensions, Features, Queue},
    image::{view::ImageView, AttachmentImage, ImageDimensions, ImageUsage},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
    sync::GpuFuture,
};

pub struct Renderer {
    app: AnimationApp,
    context: Context,
    output_file: Box<str>,

    buffer_image: Arc<ImageView<Arc<AttachmentImage>>>,
}

impl Renderer {
    pub fn new(app: AnimationApp, output_file: impl AsRef<str>) -> Self {
        ffmpeg::init().unwrap();

        let context = Context::get(
            Pixel::RGBA,
            app.screen().width(),
            app.screen().height(),
            Pixel::RGB24,
            app.screen().width(),
            app.screen().height(),
            ffmpeg::software::scaling::Flags::empty(),
        )
        .unwrap();

        let buffer_image = ImageView::new(
            AttachmentImage::with_usage(
                app.vulkan_device(),
                app.screen().0,
                vulkano::format::Format::R8G8B8A8Unorm,
                ImageUsage {
                    storage: true,
                    transfer_source: true,
                    transfer_destination: true,
                    ..ImageUsage::none()
                },
            )
            .unwrap(),
        )
        .unwrap();

        Renderer { app, context, output_file: output_file.as_ref().into(), buffer_image }
    }

    pub fn render_frame(&mut self, scene: &mut Scene) {
        let (fut, _) = self
            .app
            .app
            .render(vulkano::sync::now(self.app.vulkan_device()), scene, {
                let image = self.buffer_image.clone();
                move |_| image
            })
            .unwrap();

        let image_buf = CpuAccessibleBuffer::from_iter(
            self.app.vulkan_device(),
            BufferUsage::all(),
            false,
            (0..self.app.app.info().size_of_image_array() * 4).map(|_| 0u8),
        )
        .expect("failed to create buffer");

        let mut cmd = AutoCommandBufferBuilder::new(
            self.app.vulkan_device(),
            self.app.app.info().graphics_queue.family(),
        )
        .unwrap();
        cmd.copy_image_to_buffer(self.buffer_image.image().clone(), image_buf.clone()).unwrap();
        let cmd = cmd.build().unwrap();

        let copy_fut = fut.then_execute(self.app.app.info().graphics_queue.clone(), cmd).unwrap();

        copy_fut.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
        let mut content = image_buf.read().unwrap();

        let image = image::ImageBuffer::<Rgba<u8>, _>::from_raw(
            self.app.screen().width(),
            self.app.screen().height(),
            &content[..],
        )
        .unwrap();
        image.save(Path::new(self.output_file.as_ref())).unwrap();
    }
}

pub struct AnimationApp {
    app: App,
    fps: u32,
}

impl AnimationApp {
    pub fn new(screen: Screen, fps: u32) -> Self {
        let instance = init_instance();
        let app = init_app(instance, screen);

        Self { app, fps }
    }
    pub fn vulkan_device(&self) -> Arc<Device> {
        self.app.info().device.clone()
    }
    pub fn screen(&self) -> Screen {
        self.app.info().screen.clone()
    }
}

fn init_device_and_queue(instance: &Arc<Instance>) -> (Arc<Device>, Arc<Queue>) {
    #[cfg(debug_assertions)]
    PhysicalDevice::enumerate(&instance).for_each(|d| {
        println!("Device available: {}", d.name());
    });

    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let family = physical.queue_families().find(|&q| q.supports_compute()).unwrap();

    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions { khr_storage_buffer_storage_class: true, ..DeviceExtensions::none() },
        std::iter::once((family, 1.0)),
    )
    .unwrap();

    let graphics_queue = queues.next().unwrap();

    (device, graphics_queue)
}

fn init_instance() -> Arc<Instance> {
    Instance::new(None, &InstanceExtensions::none(), None).unwrap()
}

fn init_app(instance: Arc<Instance>, screen: Screen) -> App {
    let (device, graphics_queue) = init_device_and_queue(&instance);

    let app = AppBuilder::new(AppInfo::new(instance, graphics_queue, device.clone(), screen))
        .then_ray_tracing_pipeline()
        .then_command(Box::new(rencan_render::commands::SkyCommandFactory::new(device.clone())))
        .then_command(Box::new(rencan_render::commands::LightningV2CommandFactory::new(
            device.clone(),
            700,
        )))
        .build();

    app
}
