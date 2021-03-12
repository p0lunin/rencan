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
    light::{DirectionLight, DirectionLightUniform},
    model_buffers::SceneBuffers,
    ray::Ray,
    AppInfo, BufferAccessData, CommandFactory, CommandFactoryContext, Scene, Screen,
};
use vulkano::{
    buffer::CpuBufferPool,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferExecError},
    device::Device,
    format::ClearValue,
    instance::QueueFamily,
};
use vulkano::descriptor::{DescriptorSet, PipelineLayoutAbstract};

pub struct App {
    info: AppInfo,
    camera: Camera,
    commands: Vec<Box<dyn CommandFactory>>,
    buffers: GlobalBuffers,
}

impl App {
    pub fn new(
        info: AppInfo,
        camera: Camera,
        commands: Vec<Box<dyn CommandFactory>>,
        buffers: GlobalBuffers,
    ) -> Self {
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
        self.buffers.resize_buffers(
            &self.info.device,
            self.info.graphics_queue.family(),
            self.info.size_of_image_array(),
        );
    }
    pub fn update_camera(&mut self, update_cam: impl FnOnce(Camera) -> Camera) {
        self.camera = update_cam(self.camera.clone());
    }
    pub fn render<Prev, F>(
        &self,
        previous: Prev,
        scene: &Scene,
        image_create: F,
    ) -> Result<
        (impl GpuFuture + 'static, Arc<dyn ImageViewAccess + Send + Sync + 'static>),
        CommandBufferExecError,
    >
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
            camera: &self.camera
        };

        let mut fill_buffers = AutoCommandBufferBuilder::new(
            self.info.device.clone(),
            self.info.graphics_queue.family(),
        )
        .unwrap();

        let command = fill_buffers.build().unwrap();

        let mut fut: Box<dyn GpuFuture> =
            Box::new(previous.then_execute(self.info.graphics_queue.clone(), command).unwrap());

        for factory in self.commands.iter() {
            let command = factory.make_command(ctx.clone());
            let f = fut.then_execute_same_queue(command)?;
            fut = Box::new(f);
        }

        Ok((fut, image))
    }
    fn create_buffers(
        &self,
        image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
        scene: &Scene,
    ) -> Buffers {
        self.buffers.make_buffers(
            self.info.device.clone(),
            &self.info,
            &self.camera,
            image,
            &scene.global_light,
            &scene
        )
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
                std::iter::once(family.clone()),
            )
            .unwrap(),
            intersections: DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage::all(),
                std::iter::once(family.clone()),
            )
            .unwrap(),
            camera: Arc::new(CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer())),
            screen: Arc::new(CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer())),
            direction_light: Arc::new(CpuBufferPool::new(
                device.clone(),
                BufferUsage::uniform_buffer(),
            )),
        }
    }

    pub fn global_app_buffers(&self) -> GlobalAppBuffers {
        GlobalAppBuffers { rays: self.rays.clone(), intersections: self.intersections.clone() }
    }

    pub fn make_buffers(
        &self,
        device: Arc<Device>,
        app: &AppInfo,
        camera: &Camera,
        image: Arc<dyn ImageViewAccess + Send + Sync + 'static>,
        light: &DirectionLight,
        scene: &Scene,
    ) -> Buffers {
        Buffers::new(
            device,
            self.rays.clone(),
            self.intersections.clone(),
            Arc::new(self.camera.next(camera.clone().into_uniform().as_std140()).unwrap()),
            Arc::new(self.screen.next(app.screen.clone()).unwrap()),
            image,
            Arc::new(
                self.direction_light.next(light.clone().into_uniform()).unwrap(),
            ),
            scene.frame_buffers(),
        )
    }

    pub fn resize_buffers(&mut self, device: &Arc<Device>, family: QueueFamily, new_size: usize) {
        self.rays = DeviceLocalBuffer::array(
            device.clone(),
            new_size,
            BufferUsage::all(),
            std::iter::once(family.clone()),
        )
        .unwrap();
        self.intersections = DeviceLocalBuffer::array(
            device.clone(),
            new_size,
            BufferUsage::all(),
            std::iter::once(family.clone()),
        )
        .unwrap();
    }
}

#[derive(Clone)]
pub struct Buffers {
    pub rays: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    pub intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    pub global_app_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub models_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub lights_set: Arc<dyn DescriptorSet + Send + Sync>,
}
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

impl Buffers {
    pub fn new(
        device: Arc<Device>,
        rays: Arc<DeviceLocalBuffer<[Ray]>>,
        intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
        camera: Arc<
            dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type> + Send + Sync,
        >,
        screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync>,
        output_image: Arc<dyn ImageViewAccess + Send + Sync>,
        direction_light: Arc<
            dyn BufferAccessData<Data = DirectionLightUniform> + Send + Sync + 'static,
        >,
        models_buffers: SceneBuffers,
    ) -> Self {

        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                path: "../rencan-render/shaders/lightning.glsl"
            }
        }

        let shader = cs::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            vulkano::pipeline::ComputePipeline::new(device, &shader.main_entry_point(), &(), None).unwrap(),
        );

        let global_app_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.layout().descriptor_set_layout(0).unwrap().clone())
                .add_buffer(screen.clone())
                .unwrap()
                .add_buffer(camera.clone())
                .unwrap()
                .add_buffer(rays.clone())
                .unwrap()
                .add_buffer(intersections.clone())
                .unwrap()
                .add_image(output_image.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let models_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.layout().descriptor_set_layout(1).unwrap().clone())
                .add_buffer(models_buffers.count.clone())
                .unwrap()
                .add_buffer(models_buffers.infos.clone())
                .unwrap()
                .add_buffer(models_buffers.vertices.clone())
                .unwrap()
                .add_buffer(models_buffers.indices.clone())
                .unwrap()
                .add_buffer(models_buffers.hit_boxes.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let lights_set = Arc::new(
            PersistentDescriptorSet::start(pipeline.layout().descriptor_set_layout(2).unwrap().clone())
                .add_buffer(direction_light)
                .unwrap()
                .add_buffer(models_buffers.point_lights_count.clone())
                .unwrap()
                .add_buffer(models_buffers.point_lights.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        Buffers { rays, intersections, global_app_set, models_set, lights_set }
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
    pub fn then_command(
        mut self,
        f: Box<dyn CommandFactory>,
    ) -> Self {
        self.commands.push(f);
        self
    }
    pub fn build(self) -> App {
        App::new(self.info, self.camera, self.commands, self.global_buffers)
    }
}
