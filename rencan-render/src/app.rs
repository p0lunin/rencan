use crate::{
    camera::{Camera, CameraUniform},
    commands,
    core::CommandFactoryContext,
};
use crevice::std140::AsStd140;
use nalgebra::Point4;
use rencan_core::{AppInfo, BufferAccessData, CommandFactory, Model, Screen};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    descriptor::{descriptor_set::UnsafeDescriptorSetLayout, DescriptorSet},
    image::ImageViewAccess,
    sync::GpuFuture,
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
                path: "shaders/ray_tracing.glsl"
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
    pub fn new(info: AppInfo, camera: Camera) -> Self {
        let commands: Vec<Box<dyn CommandFactory>> = vec![
            Box::new(commands::ComputeRaysCommandFactory::new(info.device.clone())),
            Box::new(commands::RayTraceCommandFactory::new(info.device.clone())),
        ];
        Self { info, camera, commands }
    }
    pub fn info(&self) -> &AppInfo {
        &self.info
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
    fn into_descriptor_set(self, layout: Arc<UnsafeDescriptorSetLayout>) -> AppDescriptorSet {
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

/* TODO: api?
pub struct AppBuilder<'a> {
    instance: Option<Arc<Instance>>,
    physical: Option<Arc<>>
}

impl AppBuilder {
    pub fn new() -> Self {
        unimplemented!()
    }
    pub fn instance(mut self, i: Arc<Instance>) -> Self {
        self.instance = Some(i);
        self
    }
    pub fn instance_with_ext(mut self, ext: &InstanceExtensions) -> Self {
        self.instance = Some(Instance::new(
            None,
            ext,
            None,
        ).unwrap());
        self
    }
    pub fn default_instance(mut self) -> Self {
        self.instance_with_ext(&InstanceExtensions::none())
    }
    pub fn device(mut self, f: impl Fn())
}
*/
