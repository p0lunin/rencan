use std::sync::Arc;

use vulkano::{
    command_buffer::AutoCommandBufferBuilder,
    device::{Device, DeviceExtensions, Features, Queue},
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
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use rencan_core::{AppInfo, Scene, Screen};
use rencan_render::{App, AppBuilder, AppBuilderRtExt};
use std::collections::HashSet;
use vulkano::{
    image::{view::ImageView, AttachmentImage, ImageAccess},
    swapchain::SupportedPresentModes,
};

pub struct GuiApp {
    app: App,
    present_queue: Arc<Queue>,
    surface: Arc<Surface<Window>>,
    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    must_recreate_swapchain: bool,
    prev: Option<Box<dyn GpuFuture>>,

    buffer_image: Arc<ImageView<Arc<AttachmentImage>>>,
}

impl GuiApp {
    pub fn new(window: WindowBuilder, event_loop: &EventLoop<()>) -> Self {
        let instance = init_vulkan();
        let surface = window.build_vk_surface(event_loop, instance.clone()).unwrap();
        let screen = Screen(surface.window().inner_size().into());
        let (app, present_queue) = init_app(&surface, instance, screen);
        let (swap_chain, images) =
            init_swapchain(&surface, app.info().device.clone(), app.info().graphics_queue.clone());
        let prev =
            Some(Box::new(vulkano::sync::now(app.info().device.clone())) as Box<dyn GpuFuture>);
        let buffer_image = ImageView::new(
            AttachmentImage::with_usage(
                app.info().device.clone(),
                swap_chain.dimensions(),
                swap_chain.format(),
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

        GuiApp {
            app,
            present_queue,
            surface,
            swap_chain,
            swap_chain_images: images,
            must_recreate_swapchain: false,
            prev,
            buffer_image,
        }
    }

    pub fn reacreate_swapchain(&mut self) {
        self.must_recreate_swapchain = true;
    }

    pub fn render_frame(&mut self, scene: &mut Scene) {
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
            self.buffer_image = ImageView::new(
                AttachmentImage::with_usage(
                    self.device(),
                    self.swap_chain.dimensions(),
                    self.swap_chain.format(),
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

        let fut = self.prev.take().unwrap().join(acquire_future);

        let swapchain_image = self.swap_chain_images[image_num].clone();
        let (fut, _) = self
            .app
            .render(fut, scene, {
                let image = self.buffer_image.clone();
                move |_| image
            })
            .unwrap();

        let mut copy_command =
            AutoCommandBufferBuilder::new(self.device(), self.graphics_queue().family()).unwrap();
        let extent = self.buffer_image.image().dimensions().width_height_depth();
        copy_command
            .copy_image(
                self.buffer_image.image().clone(),
                [0, 0, 0],
                0,
                0,
                swapchain_image,
                [0, 0, 0],
                0,
                0,
                extent,
                1,
            )
            .unwrap();
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
        self.present_queue.clone()
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
        caps.max_image_count.unwrap_or(caps.min_image_count + 2),
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
        choose_swap_present_mode(caps.present_modes),
        FullscreenExclusive::Default,
        true,
        ColorSpace::SrgbNonLinear,
    )
    .unwrap()
}

fn init_app(
    window: &Arc<Surface<Window>>,
    instance: Arc<Instance>,
    screen: Screen,
) -> (App, Arc<Queue>) {
    let (device, graphics_queue, present_queue) = init_device_and_queues(window, &instance);

    let app = AppBuilder::new(AppInfo::new(instance, graphics_queue, device.clone(), screen))
        .then_ray_tracing_pipeline()
        .then_command(Box::new(rencan_render::commands::SkyCommandFactory::new(device.clone())))
        .then_command(Box::new(rencan_render::commands::LightningV2CommandFactory::new(
            device.clone(),
            1,
        )))
        .build();

    (app, present_queue)
}

fn init_vulkan() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();
    Instance::new(None, &extensions, None).unwrap()
}

fn init_device_and_queues(
    window: &Arc<Surface<Window>>,
    instance: &Arc<Instance>,
) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let indices = find_queue_families(window, &physical);
    let families = [indices.graphics_family, indices.present_family];
    use std::iter::FromIterator;
    let unique_queue_families: HashSet<&i32> = HashSet::from_iter(families.iter());

    let queue_priority = 1.0;
    let queue_families = unique_queue_families
        .iter()
        .map(|i| (physical.queue_families().nth(**i as usize).unwrap(), queue_priority));
    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions {
            khr_swapchain: true,
            khr_storage_buffer_storage_class: true,
            ..DeviceExtensions::none()
        },
        queue_families,
    )
    .unwrap();

    let graphics_queue = queues.next().unwrap();
    let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

    (device, graphics_queue, present_queue)
}

fn choose_swap_present_mode(available_present_modes: SupportedPresentModes) -> PresentMode {
    if available_present_modes.mailbox {
        PresentMode::Mailbox
    } else if available_present_modes.immediate {
        PresentMode::Immediate
    } else {
        PresentMode::Fifo
    }
}

fn find_queue_families(
    surface: &Arc<Surface<Window>>,
    device: &PhysicalDevice,
) -> QueueFamilyIndices {
    let mut indices = QueueFamilyIndices::new();

    for (i, queue_family) in device.queue_families().enumerate() {
        if queue_family.supports_compute() {
            indices.graphics_family = i as i32;
        }

        if surface.is_supported(queue_family).unwrap() {
            indices.present_family = i as i32;
        }

        if indices.is_complete() {
            break;
        }
    }

    indices
}

struct QueueFamilyIndices {
    graphics_family: i32,
    present_family: i32,
}
impl QueueFamilyIndices {
    fn new() -> Self {
        Self { graphics_family: -1, present_family: -1 }
    }

    fn is_complete(&self) -> bool {
        self.graphics_family >= 0 && self.present_family >= 0
    }
}
