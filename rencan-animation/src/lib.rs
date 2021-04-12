use ffmpeg::{format::Pixel, software::scaling::Context, StreamMut};
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
use ffmpeg::format::context::Output;
use std::ops::Deref;
use ffmpeg::util::frame::Video;
use itertools::Itertools;
use ffmpeg::codec::traits::Encoder;
use std::io::Write;

pub struct FFMpeg {
    context: ffmpeg::format::context::Output,
    scale_context: ffmpeg::software::scaling::context::Context,
    video_rgb: Video,
    video_yuv: Video,
    encoder: ffmpeg::codec::encoder::video::Encoder,
    frame_number: u32,
    fps: i32,
}
impl FFMpeg {
    pub fn new(output_file: impl AsRef<str>, width: u32, height: u32, fps: i32) -> Self {
        ffmpeg::init().unwrap();

        let scale_context = ffmpeg::software::scaling::context::Context::get(
            Pixel::RGBA,
            width,
            height,
            Pixel::YUV420P,
            width,
            height,
            ffmpeg::software::scaling::Flags::empty(),
        )
        .unwrap();

        let mut options = ffmpeg::Dictionary::new();
        options.set("preset", "slow");
        options.set("crf", "20");

        let mut output = ffmpeg::format::output(&output_file.as_ref()).unwrap();

        let codec = ffmpeg::codec::encoder::find_by_name("libx264").unwrap();

        let mut stream = output.add_stream(codec.clone()).unwrap();
        stream.set_time_base((1, fps));

        let mut encoder = {
            let mut video = ffmpeg::codec::encoder::video::Video(
                ffmpeg::codec::encoder::encoder::Encoder(
                    stream.codec()
                )
            );
            video.set_time_base((1, fps));
            video.set_width(width);
            video.set_height(height);
            video.set_format(
                Pixel::YUV420P
            );
            video.open_as(codec.clone()).unwrap()
        };

        ffmpeg::format::context::output::dump(&output, 0, Some(&output_file.as_ref()));

        output.write_header().unwrap();

        let video_rgb = ffmpeg::util::frame::video::Video::new(
            ffmpeg::format::Pixel::RGBA,
            width,
            height,
        );
        let video_yuv = ffmpeg::util::frame::video::Video::new(
            ffmpeg::format::Pixel::YUV420P,
            width,
            height,
        );

        Self {
            context: output,
            scale_context,
            video_rgb,
            video_yuv,
            frame_number: 0,
            encoder,
            fps,
        }
    }
    pub fn write_frame(&mut self, frame_rgba: &[u8]) {
        let mut video_rgb = &mut self.video_rgb;
        let mut video_yuv = &mut self.video_yuv;

        let frame_data = video_rgb.plane_mut::<[u8; 4]>(0);
        let mut i = 0;
        for (r, g, b, a) in frame_rgba.iter().tuples() {
            frame_data[i][0] = *r;
            frame_data[i][1] = *g;
            frame_data[i][2] = *b;
            frame_data[i][3] = *a;
            i += 1;
        }
        self.scale_context.run(video_rgb, video_yuv).unwrap();
        let mut packet = ffmpeg::codec::packet::packet::Packet::empty();

        self.frame_number += 1;
        video_yuv.set_pts(Some(self.frame_number as i64));

        let mut encoder = &mut self.encoder;
        let got_output = encoder.encode(video_yuv, &mut packet).unwrap();
        if got_output {
            packet.rescale_ts((1, self.fps), self.stream().time_base());
            packet.set_stream(0);
            println!("Writes frame {} with size {}", self.frame_number, packet.size());
            packet.write_interleaved(&mut self.context).unwrap();
        }
    }
    fn end(&mut self) {
        let mut packet = ffmpeg::codec::packet::packet::Packet::empty();
        println!("Writing delayed frames");
        loop {
            let got_output = self.encoder.flush(&mut packet).unwrap();
            if got_output {
                packet.rescale_ts((1, self.fps), self.stream().time_base());
                packet.set_stream(0);
                println!("Writes frame {} with size {}", packet.pts().unwrap(), packet.size());
                packet.write_interleaved(&mut self.context).unwrap();
            }
            else {
                break;
            }
            self.frame_number += 1;
        }
        self.context.write_trailer().unwrap();
    }
    fn stream_codec(&self) -> ffmpeg::codec::Context {
        self.stream().codec()
    }
    fn stream(&self) -> ffmpeg::format::stream::Stream {
         self.context.stream(0).unwrap()
    }
}

pub struct Renderer {
    app: AnimationApp,
    ffmpeg: FFMpeg,
    output_file: Box<str>,

    buffer_image: Arc<ImageView<Arc<AttachmentImage>>>,
}

impl Renderer {
    pub fn new(app: AnimationApp, fps: u32, output_file: impl AsRef<str>) -> Self {
        let ffmpeg = FFMpeg::new(output_file.as_ref(), app.screen().width(), app.screen().height(), fps as i32);

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

        Renderer { app, ffmpeg, output_file: output_file.as_ref().into(), buffer_image }
    }

    pub fn render_frame_to_video(&mut self, scene: &mut Scene) {
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

        println!("Frame rendered! Passing to ffmpeg...");

        self.ffmpeg.write_frame(&content[..]);
    }

    pub fn render_frame_to_image(&mut self, scene: &mut Scene) {
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

    pub fn end_video(&mut self) {
        self.ffmpeg.end();
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app.app
    }
}

pub struct AnimationApp {
    app: App,
}

impl AnimationApp {
    pub fn new(screen: Screen) -> Self {
        let instance = init_instance();
        let app = init_app(instance, screen);

        Self { app }
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
            0,
        )))
        .build();

    app
}
