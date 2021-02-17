use std::sync::Arc;

use crevice::std140::AsStd140;
use nalgebra::Point4;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    descriptor::{descriptor_set::UnsafeDescriptorSetLayout, DescriptorSet},
    image::ImageViewAccess,
    sync::GpuFuture,
};

use crate::{
    camera::{Camera, CameraUniform},
    AppInfo, BufferAccessData, CommandFactory, CommandFactoryContext, Model, Screen,
};

pub struct App {
    info: AppInfo,
    camera: Camera,
    commands: Vec<Box<dyn CommandFactory>>,
}

macro_rules! get_layout {
    ($this:expr, $to:ident) => {
        use vulkano::{descriptor::PipelineLayoutAbstract, pipeline::ComputePipeline};
        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                path: "../rencan-render/shaders/ray_tracing.glsl"
            }
        }
        let shader = cs::Shader::load($this.info.device.clone()).unwrap();
        let compute_pipeline =
            ComputePipeline::new($this.info.device.clone(), &shader.main_entry_point(), &(), None)
                .unwrap();
        $to = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
    };
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
        models: &[Model],
        image_create: F,
    ) -> (impl GpuFuture, Arc<dyn ImageViewAccess + Send + Sync + 'static>)
    where
        Prev: GpuFuture + 'static,
        F: FnOnce(&AppInfo) -> Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    {
        let image = image_create(&self.info);
        let buffers = self.create_buffers(image.clone());
        let layout;
        get_layout!(self, layout);
        let set = buffers.into_descriptor_set(layout.clone());
        let ctx = CommandFactoryContext {
            app_info: &self.info,
            global_set: set.clone().into_inner(),
            count_of_workgroups: (self.info.size_of_image_array() / 64) as u32,
            models: models.clone(),
        };
        let mut fut: Box<dyn GpuFuture> = Box::new(previous);

        for factory in self.commands.iter() {
            let command = factory.make_command(ctx.clone());
            fut = Box::new(fut.then_execute_same_queue(command).unwrap());
        }

        (fut, image)
    }
    fn create_buffers(&self, image: Arc<dyn ImageViewAccess + Send + Sync + 'static>) -> Buffers {
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
        Buffers { rays, camera, screen, output_image: image }
    }
}

pub struct Buffers {
    rays: Arc<dyn BufferAccessData<Data = Rays> + Send + Sync + 'static>,
    camera: Arc<
        dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type>
            + Send
            + Sync
            + 'static,
    >,
    screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync + 'static>,
    output_image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
}

impl Buffers {
    pub fn new(
        rays: Arc<dyn BufferAccessData<Data = [Point4<f32>]> + Send + Sync>,
        camera: Arc<
            dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type> + Send + Sync,
        >,
        screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync>,
        output_image: Arc<dyn ImageViewAccess + Send + Sync>,
    ) -> Self {
        Buffers { rays, camera, screen, output_image }
    }
}

impl Buffers {
    pub fn into_descriptor_set(self, layout: Arc<UnsafeDescriptorSetLayout>) -> AppDescriptorSet {
        use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

        let Buffers { rays, camera, screen, output_image } = self;

        AppDescriptorSet(Arc::new(
            PersistentDescriptorSet::start(layout)
                .add_buffer(screen)
                .unwrap()
                .add_buffer(camera)
                .unwrap()
                .add_buffer(rays)
                .unwrap()
                .add_image(output_image)
                .unwrap()
                .build()
                .unwrap(),
        ))
    }
}

#[derive(Clone)]
pub struct AppDescriptorSet(Arc<dyn DescriptorSet + Send + Sync>);
impl AppDescriptorSet {
    pub fn into_inner(self) -> Arc<dyn DescriptorSet + Send + Sync> {
        self.0
    }
}
pub type Rays = [Point4<f32>];

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
