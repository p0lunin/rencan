use std::collections::HashSet;
use std::convert::identity;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::{
    layers_list, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, PhysicalDeviceType,
    QueueFamily, Version,
};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain::{
    acquire_next_image, Capabilities, ColorSpace, CompositeAlpha, FullscreenExclusive, PresentMode,
    SupportedPresentModes, Surface, Swapchain,
};
use vulkano::sync::{GpuFuture, SharingMode};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

type Layers = &'static [&'static str];

pub struct Application<GraphicsPipeline> {
    validation_layers: Layers,
    debug_callback: Option<DebugCallback>,
    instance: Arc<Instance>,
    physical_device_index: usize,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    swap_chain_framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    command_buffers: Vec<Arc<AutoCommandBuffer>>,
}

impl<GraphicsPipeline> Application<GraphicsPipeline> {
    pub fn new(
        validation_layers: Layers,
        window: WindowBuilder,
        event_loop: &EventLoop<()>,
        create_rp: impl Fn(&Arc<Device>, Format) -> Arc<dyn RenderPassAbstract + Send + Sync>,
        create_gp: impl Fn(
            &Arc<Device>,
            [u32; 2],
            &Arc<dyn RenderPassAbstract + Send + Sync>,
        ) -> Arc<GraphicsPipeline>,
    ) -> Self {
        let instance = init_vulkan(validation_layers);
        let surface =
            VkSurfaceBuild::build_vk_surface(window, event_loop, instance.clone()).unwrap();
        let debug_callback = setup_debug_callback(&instance);
        let physical_device_index = pick_physical_device(&surface, &instance);
        let (device, graphics_queue, present_queue) =
            create_logical_device(&surface, &instance, physical_device_index);
        let (swap_chain, swap_chain_images) = create_swap_chain(
            &instance,
            &surface,
            physical_device_index,
            &device,
            &graphics_queue,
            &present_queue,
        );

        let render_pass = create_rp(&device, swap_chain.format());
        let graphics_pipeline = create_gp(&device, swap_chain.dimensions(), &render_pass);

        let swap_chain_framebuffers = create_framebuffers(&swap_chain_images, &render_pass);

        Application {
            validation_layers,
            debug_callback,
            instance,
            physical_device_index,
            device,
            surface,
            graphics_queue,
            present_queue,
            swap_chain,
            swap_chain_images,
            render_pass,
            graphics_pipeline,
            swap_chain_framebuffers,
            command_buffers: vec![],
        }
    }

    pub fn add_command_foreach_framebuffer<F>(&mut self, create: F) -> &mut Self
    where
        F: Fn(
            &Arc<Device>,
            QueueFamily,
            &Arc<dyn FramebufferAbstract + Send + Sync>,
            &Arc<GraphicsPipeline>,
        ) -> Arc<AutoCommandBuffer>,
    {
        let family = self.graphics_queue.family();
        let device = &self.device;
        let pipeline = &self.graphics_pipeline;
        let commands = self
            .swap_chain_framebuffers
            .iter()
            .map(|framebuffer| create(device, family, framebuffer, pipeline))
            .collect::<Vec<_>>();
        self.command_buffers.extend(commands);
        self
    }

    fn draw_frame(&mut self) {
        let (index, is_optimal, future) =
            acquire_next_image(self.swap_chain.clone(), None).unwrap();
        //assert!(is_optimal);

        let command = self.command_buffers[index].clone();

        let future = future
            .then_execute(self.graphics_queue.clone(), command)
            .unwrap()
            .then_swapchain_present(self.present_queue.clone(), self.swap_chain.clone(), index)
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    pub fn run(mut self, event_loop: EventLoop<()>)
    where
        GraphicsPipeline: 'static,
    {
        event_loop.run(move |event, _target, flow| {
            *flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(5));
            self.draw_frame();

            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                *flow = ControlFlow::Exit;
            }
        });
    }
}

struct QueueFamilyIndices {
    graphics_family: i32,
    present_family: i32,
}
impl QueueFamilyIndices {
    fn new() -> Self {
        Self {
            graphics_family: -1,
            present_family: -1,
        }
    }

    fn is_complete(&self) -> bool {
        self.graphics_family >= 0 && self.present_family >= 0
    }
}

fn pick_physical_device(surface: &Arc<Surface<Window>>, instance: &Arc<Instance>) -> usize {
    PhysicalDevice::enumerate(instance)
        .position(|device| is_suitable(surface, &device))
        .expect("Failed to find a suitable GPU")
}

fn is_suitable(surface: &Arc<Surface<Window>>, device: &PhysicalDevice) -> bool {
    let indices = find_queue_families(surface, device);
    let extensions_supported = check_device_extensions_support(device);

    let swap_chain_adequate = if extensions_supported {
        let capabilities = surface
            .capabilities(*device)
            .expect("failed to get surface capabilities");
        !capabilities.supported_formats.is_empty()
            && capabilities.present_modes.iter().next().is_some()
    } else {
        false
    };

    indices.is_complete() && extensions_supported && swap_chain_adequate
}

fn find_queue_families(
    surface: &Arc<Surface<Window>>,
    device: &PhysicalDevice,
) -> QueueFamilyIndices {
    let mut indices = QueueFamilyIndices::new();
    for (i, queue_family) in device.queue_families().enumerate() {
        if queue_family.supports_graphics() {
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

fn create_swap_chain(
    instance: &Arc<Instance>,
    surface: &Arc<Surface<Window>>,
    physical_device_index: usize,
    device: &Arc<Device>,
    graphics_queue: &Arc<Queue>,
    present_queue: &Arc<Queue>,
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
    let physical = PhysicalDevice::from_index(instance, physical_device_index).unwrap();
    let capabilities = surface
        .capabilities(physical)
        .expect("failed to load capabilities");

    let surface_format = choose_swap_surface_format(&capabilities.supported_formats);
    let present_mode = choose_swap_present_mode(capabilities.present_modes);
    let extent = choose_swap_extent(surface.window().inner_size().into(), &capabilities);

    let mut image_count = capabilities.min_image_count + 1;

    if let Some(max_count) = capabilities.max_image_count {
        if image_count > max_count {
            image_count = max_count;
        }
    }

    let image_usage = ImageUsage {
        color_attachment: true,
        ..ImageUsage::none()
    };

    let indices = find_queue_families(&surface, &physical);

    let sharing: SharingMode = if indices.graphics_family != indices.present_family {
        vec![graphics_queue, present_queue].as_slice().into()
    } else {
        graphics_queue.into()
    };

    let (swap_chain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        image_count,
        surface_format.0,
        extent,
        1,
        image_usage,
        sharing,
        capabilities.current_transform,
        CompositeAlpha::Opaque,
        present_mode,
        FullscreenExclusive::Allowed,
        true,
        surface_format.1,
    )
    .expect("Error when creating swapchain");

    (swap_chain, images)
}

fn choose_swap_surface_format(available_formats: &[(Format, ColorSpace)]) -> (Format, ColorSpace) {
    *available_formats
        .iter()
        .find(|(format, color_space)| {
            *format == Format::B8G8R8A8Unorm && *color_space == ColorSpace::SrgbNonLinear
        })
        .unwrap_or_else(|| &available_formats[0])
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

fn choose_swap_extent(window_size: [u32; 2], capabilities: &Capabilities) -> [u32; 2] {
    capabilities.current_extent.unwrap_or_else(|| {
        let mut actual = window_size;
        actual[0] =
            capabilities.min_image_extent[0].max(capabilities.min_image_extent[0].min(actual[0]));

        actual
    })
}

fn create_logical_device(
    surface: &Arc<Surface<Window>>,
    instance: &Arc<Instance>,
    physical: usize,
) -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
    let physical = PhysicalDevice::from_index(instance, physical)
        .expect("We previously get index from Vulkan");
    let indices = find_queue_families(surface, &physical);

    let families = [indices.graphics_family, indices.present_family];
    use std::iter::FromIterator;
    let unique_queue_families: HashSet<&i32> = HashSet::from_iter(families.iter());

    let priority = 1.0;
    let families = unique_queue_families.iter().map(|i| {
        (
            physical.queue_families().nth(**i as usize).unwrap(),
            priority,
        )
    });

    let (device, mut queues) =
        Device::new(physical, &Features::none(), &device_extensions(), families)
            .expect("Failed to create device");

    let graphics = queues.next().unwrap();
    let present = queues.next().unwrap_or_else(|| graphics.clone());

    (device, graphics, present)
}

fn create_framebuffers(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    images
        .iter()
        .map(|image| {
            let fba = Framebuffer::start(render_pass.clone())
                .add(image.clone())
                .unwrap()
                .build()
                .unwrap();
            Arc::new(fba) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect()
}

fn init_vulkan(layers: Layers) -> Arc<Instance> {
    let supported_extensions =
        InstanceExtensions::supported_by_core().expect("failed to retrieve supported extensions");
    println!("Supported extensions: {:?}", supported_extensions);
    let app_info = ApplicationInfo {
        application_name: Some("rencan".into()),
        application_version: Some(Version {
            major: 1,
            minor: 0,
            patch: 0,
        }),
        engine_name: Some("No Engine".into()),
        engine_version: Some(Version {
            major: 1,
            minor: 0,
            patch: 0,
        }),
    };

    let required_extensions = get_required_extensions();

    if ENABLE_VALIDATION_LAYERS && check_validation_layer_support(layers) {
        Instance::new(
            Some(&app_info),
            &required_extensions,
            layers.iter().map(|&s| s),
        )
    } else {
        Instance::new(Some(&app_info), &required_extensions, None)
    }
    .expect("failed to create Vulkan instance")
}

fn get_required_extensions() -> InstanceExtensions {
    let mut extensions = vulkano_win::required_extensions();
    if ENABLE_VALIDATION_LAYERS {
        extensions.ext_debug_utils = true;
    }

    extensions
}

fn check_device_extensions_support(device: &PhysicalDevice) -> bool {
    let available = DeviceExtensions::supported_by_device(*device);
    let exts = device_extensions();

    available.intersection(&exts) == exts
}

fn device_extensions() -> DeviceExtensions {
    DeviceExtensions {
        khr_swapchain: true,
        ..vulkano::device::DeviceExtensions::none()
    }
}

fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
    if !ENABLE_VALIDATION_LAYERS {
        return None;
    }

    let msg_severity = MessageSeverity {
        error: true,
        warning: true,
        information: false,
        verbose: true,
    };
    let msg_types = MessageType::all();
    DebugCallback::new(&instance, msg_severity, msg_types, |msg| {
        println!("validation layer: {:?}", msg.description);
    })
    .ok()
}

fn check_validation_layer_support(layers: Layers) -> bool {
    let available = layers_list().expect("failed to get layers list");
    let mut supported = vec![false; layers.len()];
    available.for_each(
        |avail| match layers.iter().position(|&s| s == avail.name()) {
            Some(idx) => supported[idx] = true,
            None => {}
        },
    );

    supported.into_iter().all(identity)
}
