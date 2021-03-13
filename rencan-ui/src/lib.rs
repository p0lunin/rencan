use std::sync::Arc;

use vulkano::{
    command_buffer::AutoCommandBufferBuilder,
    device::{Device, DeviceExtensions, Features, Queue},
    format::ClearValue,
    image::{ImageUsage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    swapchain::{
        AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
        Swapchain, SwapchainCreationError,
    },
    sync::GpuFuture,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    dpi::{PhysicalSize, Size},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use rencan_core::{camera::Camera, AppInfo, Scene, Screen};
use rencan_render::{App, AppBuilder};
use vulkano::image::AttachmentImage;

pub struct GuiApp {
    app: App,
    surface: Arc<Surface<Window>>,
    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    must_recreate_swapchain: bool,
    prev: Option<Box<dyn GpuFuture>>,
}

impl GuiApp {
    pub fn new(window: WindowBuilder, event_loop: &EventLoop<()>) -> Self {
        let instance = init_vulkan();
        let surface = window
            .with_inner_size(Size::Physical(PhysicalSize::new(512, 512)))
            .build_vk_surface(event_loop, instance.clone())
            .unwrap();
        let screen = Screen::new(512, 512);
        let app = init_app(instance, screen);
        let (swap_chain, images) =
            init_swapchain(&surface, app.info().device.clone(), app.info().graphics_queue.clone());
        let prev =
            Some(Box::new(vulkano::sync::now(app.info().device.clone())) as Box<dyn GpuFuture>);
        GuiApp {
            app,
            surface,
            swap_chain,
            swap_chain_images: images,
            must_recreate_swapchain: false,
            prev,
        }
    }

    pub fn reacreate_swapchain(&mut self) {
        self.must_recreate_swapchain = true;
    }

    pub fn render_frame(&mut self, scene: &Scene) {
        match self.prev.as_mut() {
            Some(fut) => fut.cleanup_finished(),
            None => {}
        };

        if self.must_recreate_swapchain {
            let dimensions: [u32; 2] = self.surface.window().inner_size().into();
            let (new_swapchain, new_images) =
                match self.swap_chain.recreate_with_dimensions(dimensions) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::UnsupportedDimensions) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };

            self.swap_chain = new_swapchain;
            self.swap_chain_images = new_images;
            self.must_recreate_swapchain = false;
            self.app.update_screen(Screen(dimensions));
        }

        let (image_num, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swap_chain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.must_recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.must_recreate_swapchain = true;
        }

        let mut clear_image =
            AutoCommandBufferBuilder::new(self.device(), self.graphics_queue().family()).unwrap();
        clear_image
            .clear_color_image(
                self.swap_chain_images[image_num].clone(),
                ClearValue::Float([0.0, 0.0, 0.0, 0.0]),
            )
            .unwrap();
        let clear_image = clear_image.build().unwrap();

        let fut = self
            .prev
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue(), clear_image)
            .unwrap();

        let image = AttachmentImage::with_usage(
            self.device(),
            self.swap_chain.dimensions(),
            self.swap_chain.format(),
            ImageUsage {
                storage: true,
                transfer_source: true,
                ..ImageUsage::none()
            }
        ).unwrap();
        let swapchain_image = self.swap_chain_images[image_num].clone();
        let (fut, _) = self.app.render(fut, &scene, {
            let image = image.clone();
            move |_| image
        }).unwrap();

        let mut copy_command = AutoCommandBufferBuilder::new(
            self.device(),
            self.graphics_queue().family()
        ).unwrap();
        let extent = [image.dimensions()[0], image.dimensions()[1], 1];
        copy_command.copy_image(
            image,
            [0, 0, 0],
            0,
            0,
            swapchain_image,
            [0,0,0],
            0,
            0,
            extent,
            1
        ).unwrap();
        let copy_command = copy_command.build().unwrap();

        let fut = fut
            .then_execute(self.graphics_queue(), copy_command)
            .unwrap()
            .then_swapchain_present(self.present_queue(), self.swap_chain.clone(), image_num)
            .then_signal_fence_and_flush()
            .unwrap();

        self.prev = Some(Box::new(fut));
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
    pub fn device(&self) -> Arc<Device> {
        self.app.info().device.clone()
    }
    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.app.info().graphics_queue.clone()
    }
    pub fn present_queue(&self) -> Arc<Queue> {
        self.app.info().graphics_queue.clone()
    }
}

fn init_swapchain(
    surface: &Arc<Surface<Window>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    let caps = surface.capabilities(device.physical_device()).unwrap();

    let alpha = caps.supported_composite_alpha.iter().next().unwrap();
    println!("Supported image formats: {:?}", &caps.supported_formats);
    let format = caps.supported_formats[0].0;
    let dimensions: [u32; 2] = surface.window().inner_size().into();

    Swapchain::new(
        device.clone(),
        surface.clone(),
        caps.min_image_count,
        format,
        dimensions,
        1,
        ImageUsage {
            storage: true,
            color_attachment: true,
            transfer_destination: true,
            ..ImageUsage::none()
        },
        &queue,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        FullscreenExclusive::Default,
        true,
        ColorSpace::SrgbNonLinear,
    )
    .unwrap()
}

fn init_app(instance: Arc<Instance>, screen: Screen) -> App {
    use rencan_render::AppBuilderRtExt;
    let (device, queue) = init_device_and_queues(&instance);

    AppBuilder::new(
        AppInfo::new(instance, queue, device.clone(), screen),
        Camera::from_origin().move_at(0.0, 0.0, 5.0),
    )
    .then_ray_tracing_pipeline()
    .then_command(Box::new(rencan_render::commands::LightningCommandFactory::new(device.clone())))
    .build()
}

fn init_vulkan() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();
    Instance::new(None, &extensions, None).unwrap()
}

fn init_device_and_queues(instance: &Arc<Instance>) -> (Arc<Device>, Arc<Queue>) {
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let queue_family = physical.queue_families().find(|&q| q.supports_graphics()).unwrap();

    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions {
            khr_swapchain: true,
            khr_storage_buffer_storage_class: true,
            ..DeviceExtensions::none()
        },
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    (device, queue)
}
