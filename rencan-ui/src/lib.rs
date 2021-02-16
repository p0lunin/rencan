use rencan_core::{AppInfo, Model, Screen};
use rencan_render::{camera::Camera, App};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
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
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct GuiApp {
    app: App,
    surface: Arc<Surface<Window>>,
    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    must_recreate_swapchain: bool,
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
        GuiApp {
            app,
            surface,
            swap_chain,
            swap_chain_images: images,
            must_recreate_swapchain: false,
        }
    }

    pub fn run<T, PreCompute, Models>(
        mut self,
        event_loop: EventLoop<T>,
        mut models: Models,
        mut pre_compute: PreCompute,
    ) where
        Models: 'static,
        PreCompute: for<'a> FnMut(&Event<T>, &mut App, &'a mut Models) -> &'a [Model] + 'static,
    {
        event_loop.run(move |event, _, control_flow| {
            let models_ref = pre_compute(&event, &mut self.app, &mut models);
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(5));

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                    self.must_recreate_swapchain = true;
                }
                Event::RedrawEventsCleared => {
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
                        match vulkano::swapchain::acquire_next_image(self.swap_chain.clone(), None)
                        {
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

                    let mut clear_image = AutoCommandBufferBuilder::new(
                        self.device(),
                        self.graphics_queue().family(),
                    )
                    .unwrap();
                    clear_image
                        .clear_color_image(
                            self.swap_chain_images[image_num].clone(),
                            ClearValue::Float([0.0, 0.0, 0.0, 0.0]),
                        )
                        .unwrap();
                    let clear_image = clear_image.build().unwrap();

                    let (fut, _) = self.app.render(
                        acquire_future.then_execute(self.graphics_queue(), clear_image).unwrap(),
                        models_ref,
                        |_| self.swap_chain_images[image_num].clone(),
                    );

                    fut.then_swapchain_present(
                        self.present_queue(),
                        self.swap_chain.clone(),
                        image_num,
                    )
                    .then_signal_fence_and_flush()
                    .unwrap()
                    .wait(None)
                    .unwrap();
                }
                _ => (),
            }
        })
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
    let (device, queue) = init_device_and_queues(&instance);

    App::new(
        AppInfo::new(instance, queue, device, screen),
        Camera::from_origin().move_at(0.0, 0.0, 1.0),
    )
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
