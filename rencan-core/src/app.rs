use std::sync::Arc;

use crevice::std140::AsStd140;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    image::ImageViewAccess,
    sync::GpuFuture,
};

use crate::{
    camera::{Camera, CameraUniform},
    intersection::IntersectionUniform,
    light::DirectionLightUniform,
    ray::Ray,
    AppInfo, BufferAccessData, CommandFactory, CommandFactoryContext, Scene, Screen,
};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::format::ClearValue;
use vulkano::buffer::CpuBufferPool;
use vulkano::device::Device;
use vulkano::instance::QueueFamily;
use crate::light::DirectionLight;

pub struct App {
    info: AppInfo,
    camera: Camera,
    commands: Vec<Box<dyn CommandFactory>>,
    buffers: GlobalBuffers,
}

impl App {
    pub fn new(info: AppInfo, camera: Camera, commands: Vec<Box<dyn CommandFactory>>, buffers: GlobalBuffers) -> Self {
        Self { info, camera, commands, buffers }
    }
    pub fn info(&self) -> &AppInfo {
        &self.info
    }
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    pub fn update_screen(&mut self, screen: Screen) {
        self.info.screen = screen;
        self.buffers.resize_buffers(&self.info.device, self.info.graphics_queue.family(), self.info.size_of_image_array());
        for factory in self.commands.iter_mut() {
            factory.update_buffers(self.buffers.global_app_buffers());
        }
    }
    pub fn update_camera(&mut self, update_cam: impl FnOnce(Camera) -> Camera) {
        self.camera = update_cam(self.camera.clone());
    }
    pub fn render<Prev, F>(
        &self,
        previous: Prev,
        scene: &Scene,
        image_create: F,
    ) -> (impl GpuFuture, Arc<dyn ImageViewAccess + Send + Sync + 'static>)
    where
        Prev: GpuFuture + 'static,
        F: FnOnce(&AppInfo) -> Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    {
        let image = image_create(&self.info);
        let buffers = self.create_buffers(image.clone(), scene);
        let ctx = CommandFactoryContext {
            app_info: &self.info,
            buffers: buffers.clone(),
            count_of_workgroups: (self.info.size_of_image_array() / 64) as u32,
            scene,
        };

        let mut fill_buffers = AutoCommandBufferBuilder::new(self.info.device.clone(), self.info.graphics_queue.family())
            .unwrap();
        fill_buffers.fill_buffer(self.buffers.intersections.clone(), 0).unwrap();
        fill_buffers.fill_buffer(self.buffers.rays.clone(), 0).unwrap();

        let command = fill_buffers.build().unwrap();

        let mut fut: Box<dyn GpuFuture> = Box::new(previous.then_execute(self.info.graphics_queue.clone(), command).unwrap());

        for factory in self.commands.iter() {
            let command = factory.make_command(ctx.clone());
            fut = Box::new(fut.then_execute_same_queue(command).unwrap());
        }

        (fut, image)
    }
    fn create_buffers(
        &self,
        image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
        scene: &Scene,
    ) -> Buffers {
        self.buffers.make_buffers(&self.info, &self.camera, image, &scene.global_light)
    }
}

pub struct GlobalAppBuffers {
    pub rays: Arc<DeviceLocalBuffer<[Ray]>>,
    pub intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
}

pub struct GlobalBuffers {
    rays: Arc<DeviceLocalBuffer<[Ray]>>,
    intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    camera: Arc<CpuBufferPool<<CameraUniform as AsStd140>::Std140Type>>,
    screen: Arc<CpuBufferPool<Screen>>,
    direction_light: Arc<CpuBufferPool<DirectionLightUniform>>,
}

impl GlobalBuffers {
    pub fn new(device: &Arc<Device>, family: QueueFamily, size: usize) -> Self {
        GlobalBuffers {
            rays: DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::all(),
                std::iter::once(family.clone())
            ).unwrap(),
            intersections: DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::all(),
                std::iter::once(family.clone())
            ).unwrap(),
            camera: Arc::new(CpuBufferPool::new(
                device.clone(),
                BufferUsage::all()
            )),
            screen: Arc::new(CpuBufferPool::new(
                device.clone(),
                BufferUsage::all()
            )),
            direction_light: Arc::new(CpuBufferPool::new(
                device.clone(),
                BufferUsage::all()
            ))
        }
    }

    pub fn global_app_buffers(&self) -> GlobalAppBuffers {
        GlobalAppBuffers {
            rays: self.rays.clone(),
            intersections: self.intersections.clone()
        }
    }

    pub fn make_buffers(
        &self,
        app: &AppInfo,
        camera: &Camera,
        image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
        light: &DirectionLight,
    ) -> Buffers {
        Buffers {
            camera: Arc::new(self.camera.next(camera.clone().into_uniform().as_std140()).unwrap()),
            screen: Arc::new(self.screen.next(app.screen.clone()).unwrap()),
            output_image: image,
            direction_light: Arc::new(self.direction_light.next(light.clone().into_uniform()).unwrap())
        }
    }

    pub fn resize_buffers(&mut self, device: &Arc<Device>, family: QueueFamily, new_size: usize) {
        self.rays = DeviceLocalBuffer::array(
            device.clone(),
            new_size,
            BufferUsage::all(),
            std::iter::once(family.clone())
        ).unwrap();
        self.intersections = DeviceLocalBuffer::array(
            device.clone(),
            new_size,
            BufferUsage::all(),
            std::iter::once(family.clone())
        ).unwrap();
    }
}

#[derive(Clone)]
pub struct Buffers {
    pub camera: Arc<
        dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type>
            + Send
            + Sync
            + 'static,
    >,
    pub screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync + 'static>,
    pub output_image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    pub direction_light:
        Arc<dyn BufferAccessData<Data = DirectionLightUniform> + Send + Sync + 'static>,
}

impl Buffers {
    pub fn new(
        camera: Arc<
            dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type> + Send + Sync,
        >,
        screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync>,
        output_image: Arc<dyn ImageViewAccess + Send + Sync>,
        direction_light: Arc<
            dyn BufferAccessData<Data = DirectionLightUniform> + Send + Sync + 'static,
        >,
    ) -> Self {
        Buffers { camera, screen, output_image, direction_light }
    }
}

pub type Rays = [Ray];

pub struct AppBuilder {
    info: AppInfo,
    camera: Camera,
    commands: Vec<Box<dyn CommandFactory>>,
    global_buffers: GlobalBuffers,
}

impl AppBuilder {
    pub fn info(&self) -> &AppInfo {
        &self.info
    }
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    pub fn commands(&self) -> &Vec<Box<dyn CommandFactory>> {
        &self.commands
    }
}

impl AppBuilder {
    pub fn new(info: AppInfo, camera: Camera) -> Self {
        let buffers = GlobalBuffers::new(
            &info.device,
            info.graphics_queue.family(),
            info.size_of_image_array(),
        );
        Self { info, camera, commands: vec![], global_buffers: buffers }
    }
    pub fn then_command(mut self, f: impl FnOnce(GlobalAppBuffers) -> Box<dyn CommandFactory>) -> Self {
        self.commands.push(f(self.global_buffers.global_app_buffers()));
        self
    }
    pub fn build(self) -> App {
        App::new(self.info, self.camera, self.commands, self.global_buffers)
    }
}
