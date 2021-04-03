use std::sync::Arc;

use crevice::std140::AsStd140;
use vulkano::{
    buffer::{BufferUsage, DeviceLocalBuffer},
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
use once_cell::sync::OnceCell;
use vulkano::{
    buffer::{cpu_pool::CpuBufferPoolChunk, CpuBufferPool},
    command_buffer::{CommandBufferExecError, DispatchIndirectCommand},
    descriptor::{
        descriptor_set::FixedSizeDescriptorSetsPool, pipeline_layout::PipelineLayout,
        DescriptorSet, PipelineLayoutAbstract,
    },
    device::Device,
    image::{view::ImageView, AttachmentImage},
    instance::QueueFamily,
    memory::pool::StdMemoryPool,
    pipeline::ComputePipeline,
};

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
        &mut self,
        previous: Prev,
        scene: &Scene,
        image_create: F,
    ) -> Result<
        (impl GpuFuture + 'static, Arc<ImageView<Arc<AttachmentImage>>>),
        CommandBufferExecError,
    >
    where
        Prev: GpuFuture + 'static,
        F: FnOnce(&AppInfo) -> Arc<ImageView<Arc<AttachmentImage>>>,
    {
        let image = image_create(&self.info);
        let buffers = self.create_buffers(image.clone(), scene);
        let ctx = CommandFactoryContext {
            app_info: &self.info,
            buffers: buffers.clone(),
            scene,
            camera: &self.camera,
        };

        let mut fut: Box<dyn GpuFuture> = Box::new(previous);
        for factory in self.commands.iter_mut() {
            fut = factory.make_command(ctx.clone(), fut);
        }

        Ok((fut, image))
    }
    fn create_buffers(
        &mut self,
        image: Arc<ImageView<Arc<AttachmentImage>>>,
        scene: &Scene,
    ) -> Buffers {
        self.buffers.make_buffers(&self.info, &self.camera, image, &scene.global_light, &scene)
    }
}

pub struct GlobalAppBuffers {
    pub intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
}

pub struct GlobalBuffers {
    intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    intersections_count: Arc<CpuBufferPool<DispatchIndirectCommand>>,
    camera: Arc<CpuBufferPool<<CameraUniform as AsStd140>::Std140Type>>,
    screen: Arc<CpuBufferPool<Screen>>,
    direction_light: Arc<CpuBufferPool<DirectionLightUniform>>,

    sets: SetsStorage,
}

impl GlobalBuffers {
    pub fn new(device: &Arc<Device>, family: QueueFamily, size: usize) -> Self {
        GlobalBuffers {
            intersections: DeviceLocalBuffer::array(
                device.clone(),
                size,
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
                std::iter::once(family.clone()),
            )
            .unwrap(),
            intersections_count: Arc::new(CpuBufferPool::new(
                device.clone(),
                BufferUsage {
                    indirect_buffer: true,
                    storage_buffer: true,
                    transfer_destination: true,
                    ..BufferUsage::none()
                },
            )),
            camera: Arc::new(CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer())),
            screen: Arc::new(CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer())),
            direction_light: Arc::new(CpuBufferPool::new(
                device.clone(),
                BufferUsage::uniform_buffer(),
            )),
            sets: SetsStorage::new(device),
        }
    }

    pub fn global_app_buffers(&self) -> GlobalAppBuffers {
        GlobalAppBuffers { intersections: self.intersections.clone() }
    }

    pub fn make_buffers(
        &mut self,
        app: &AppInfo,
        camera: &Camera,
        image: Arc<ImageView<Arc<AttachmentImage>>>,
        light: &DirectionLight,
        scene: &Scene,
    ) -> Buffers {
        self.sets.buffers(
            self.intersections.clone(),
            Arc::new(
                self.intersections_count
                    .chunk(std::iter::once(DispatchIndirectCommand { x: 0, y: 0, z: 0 }))
                    .unwrap(),
            ),
            Arc::new(self.camera.next(camera.clone().into_uniform().as_std140()).unwrap()),
            Arc::new(self.screen.next(app.screen.clone()).unwrap()),
            image,
            Arc::new(self.direction_light.next(light.clone().into_uniform()).unwrap()),
            scene.frame_buffers(),
        )
    }

    pub fn resize_buffers(&mut self, device: &Arc<Device>, family: QueueFamily, new_size: usize) {
        self.intersections = DeviceLocalBuffer::array(
            device.clone(),
            new_size,
            BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            std::iter::once(family.clone()),
        )
        .unwrap();
    }
}

pub struct SetsStorage {
    pub global_app_set: FixedSizeDescriptorSetsPool,
    pub rays_set: FixedSizeDescriptorSetsPool,
    pub models_set: FixedSizeDescriptorSetsPool,
    pub sphere_models_set: FixedSizeDescriptorSetsPool,
    pub lights_set: FixedSizeDescriptorSetsPool,
    pub image_set: FixedSizeDescriptorSetsPool,
}

impl SetsStorage {
    pub fn new(device: &Arc<Device>) -> Self {
        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                path: "../rencan-render/shaders/lightning.glsl"
            }
        }

        const SHADER: OnceCell<cs::Shader> = OnceCell::new();

        const PIPELINE: OnceCell<Arc<ComputePipeline<PipelineLayout<cs::MainLayout>>>> =
            OnceCell::new();

        let pip = PIPELINE
            .get_or_init(move || {
                Arc::new(
                    vulkano::pipeline::ComputePipeline::new(
                        device.clone(),
                        &SHADER
                            .get_or_init(move || cs::Shader::load(device.clone()).unwrap())
                            .main_entry_point(),
                        &cs::SpecializationConstants { constant_0: 1, SAMPLING: 0 },
                        None,
                    )
                    .unwrap(),
                )
            })
            .clone();

        let global_app_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(0).unwrap().clone(),
        );
        let rays_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(1).unwrap().clone(),
        );
        let models_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(2).unwrap().clone(),
        );
        let sphere_models_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(3).unwrap().clone(),
        );
        let lights_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(4).unwrap().clone(),
        );
        let image_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(5).unwrap().clone(),
        );

        SetsStorage {
            global_app_set,
            rays_set,
            models_set,
            sphere_models_set,
            lights_set,
            image_set,
        }
    }

    pub fn buffers(
        &mut self,
        intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
        intersections_count: Arc<CpuBufferPoolChunk<DispatchIndirectCommand, Arc<StdMemoryPool>>>,
        camera: Arc<
            dyn BufferAccessData<Data = <CameraUniform as AsStd140>::Std140Type> + Send + Sync,
        >,
        screen: Arc<dyn BufferAccessData<Data = Screen> + Send + Sync>,
        output_image: Arc<ImageView<Arc<AttachmentImage>>>,
        direction_light: Arc<
            dyn BufferAccessData<Data = DirectionLightUniform> + Send + Sync + 'static,
        >,
        models_buffers: SceneBuffers,
    ) -> Buffers {
        let global_app_set = Arc::new(
            self.global_app_set
                .next()
                .add_buffer(screen.clone())
                .unwrap()
                .add_buffer(camera.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let models_set = Arc::new(
            self.models_set
                .next()
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

        let sphere_models_set = Arc::new(
            self.sphere_models_set
                .next()
                .add_buffer(models_buffers.sphere_count.clone())
                .unwrap()
                .add_buffer(models_buffers.sphere_infos.clone())
                .unwrap()
                .add_buffer(models_buffers.spheres.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let lights_set = Arc::new(
            self.lights_set
                .next()
                .add_buffer(direction_light)
                .unwrap()
                .add_buffer(models_buffers.point_lights_count.clone())
                .unwrap()
                .add_buffer(models_buffers.point_lights.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let rays_set = Arc::new(
            self.rays_set
                .next()
                .add_buffer(intersections.clone())
                .unwrap()
                .add_buffer(intersections_count.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let image_set = Arc::new(
            self.image_set.next().add_image(output_image.clone()).unwrap().build().unwrap(),
        );

        Buffers {
            sphere_models_set,
            image: output_image,
            workgroups: intersections_count,
            intersections,
            global_app_set,
            models_set,
            lights_set,
            rays_set,
            image_set,
        }
    }
}

#[derive(Clone)]
pub struct Buffers {
    pub intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    pub workgroups: Arc<CpuBufferPoolChunk<DispatchIndirectCommand, Arc<StdMemoryPool>>>,
    pub image: Arc<ImageView<Arc<AttachmentImage>>>,

    pub global_app_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub rays_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub models_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub sphere_models_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub lights_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub image_set: Arc<dyn DescriptorSet + Send + Sync>,
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
    pub fn then_command(mut self, f: Box<dyn CommandFactory>) -> Self {
        self.commands.push(f);
        self
    }
    pub fn build(self) -> App {
        App::new(self.info, self.camera, self.commands, self.global_buffers)
    }
}
