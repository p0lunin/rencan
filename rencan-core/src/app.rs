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

pub struct App {
    info: AppInfo,
    camera: Camera,
    commands: Vec<Box<dyn CommandFactory>>,
}

impl App {
    pub fn new(info: AppInfo, camera: Camera, commands: Vec<Box<dyn CommandFactory>>) -> Self {
        Self { info, camera, commands }
    }
    pub fn info(&self) -> &AppInfo {
        &self.info
    }
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    pub fn update_screen(&mut self, screen: Screen) {
        self.info.screen = screen;
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
            buffers,
            count_of_workgroups: (self.info.size_of_image_array() / 64) as u32,
            scene,
        };
        let mut fut: Box<dyn GpuFuture> = Box::new(previous);

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
        let rays = DeviceLocalBuffer::array(
            self.info.device.clone(),
            self.info.size_of_image_array(),
            BufferUsage::all(),
            std::iter::once(self.info.graphics_queue.family()),
        )
        .unwrap();
        let camera = CpuAccessibleBuffer::from_data(
            self.info.device.clone(),
            BufferUsage::all(),
            false,
            self.camera.clone().into_uniform().as_std140(),
        )
        .unwrap();
        let screen = CpuAccessibleBuffer::from_data(
            self.info.device.clone(),
            BufferUsage::all(),
            false,
            self.info.screen.clone(),
        )
        .unwrap();
        let intersections = DeviceLocalBuffer::array(
            self.info.device.clone(),
            self.info.size_of_image_array(),
            BufferUsage::all(),
            std::iter::once(self.info.graphics_queue.family()),
        )
        .unwrap();
        let global_light = CpuAccessibleBuffer::from_data(
            self.info.device.clone(),
            BufferUsage::all(),
            false,
            scene.global_light.clone().into_uniform(),
        )
        .unwrap();
        Buffers {
            rays,
            camera,
            screen,
            output_image: image,
            intersections,
            direction_light: global_light,
        }
    }
}

#[derive(Clone)]
pub struct Buffers {
    pub rays: Arc<dyn BufferAccessData<Data = Rays> + Send + Sync + 'static>,
    pub camera: Arc<
        dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type>
            + Send
            + Sync
            + 'static,
    >,
    pub screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync + 'static>,
    pub output_image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    pub intersections:
        Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync + 'static>,
    pub direction_light:
        Arc<dyn BufferAccessData<Data = DirectionLightUniform> + Send + Sync + 'static>,
}

impl Buffers {
    pub fn new(
        rays: Arc<dyn BufferAccessData<Data = Rays> + Send + Sync>,
        camera: Arc<
            dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type> + Send + Sync,
        >,
        screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync>,
        output_image: Arc<dyn ImageViewAccess + Send + Sync>,
        intersections: Arc<
            dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync + 'static,
        >,
        direction_light: Arc<
            dyn BufferAccessData<Data = DirectionLightUniform> + Send + Sync + 'static,
        >,
    ) -> Self {
        Buffers { rays, camera, screen, output_image, intersections, direction_light }
    }
}

pub type Rays = [Ray];

pub struct AppBuilder {
    info: AppInfo,
    camera: Camera,
    commands: Vec<Box<dyn CommandFactory>>,
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
        Self { info, camera, commands: vec![] }
    }
    pub fn then_command(mut self, factory: Box<dyn CommandFactory>) -> Self {
        self.commands.push(factory);
        self
    }
    pub fn build(self) -> App {
        App::new(self.info, self.camera, self.commands)
    }
}
