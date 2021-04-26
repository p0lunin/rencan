use std::sync::Arc;

use crevice::std140::AsStd140;
use vulkano::{
    buffer::{BufferUsage, DeviceLocalBuffer},
    sync::GpuFuture,
};

use crate::{
    camera::{CameraUniform},
    intersection::IntersectionUniform,
    light::{DirectionLightUniform},
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
use vulkano::format::ClearValue;

pub struct App {
    info: AppInfo,
    commands: Vec<Box<dyn CommandFactory>>,
    buffers: GlobalBuffers,
}

impl App {
    pub fn new(
        info: AppInfo,
        commands: Vec<Box<dyn CommandFactory>>,
        buffers: GlobalBuffers,
    ) -> Self {
        Self { info, commands, buffers }
    }
    pub fn info(&self) -> &AppInfo {
        &self.info
    }
    pub fn update_screen(&mut self, screen: Screen) {
        self.info.screen = screen;
        self.buffers.resize_buffers(
            &self.info.device,
            self.info.graphics_queue.family(),
            self.info.size_of_image_array(),
        );
    }
    pub fn render<Prev, F, AMF>(
        &mut self,
        previous: Prev,
        scene: &mut Scene,
        image_create: F,
        add_msaa: AMF,
    ) -> Result<
        (impl GpuFuture + 'static, Arc<ImageView<Arc<AttachmentImage>>>),
        CommandBufferExecError,
    >
    where
        Prev: GpuFuture + 'static,
        F: FnOnce(&AppInfo) -> Arc<ImageView<Arc<AttachmentImage>>>,
        AMF: FnOnce(Box<dyn GpuFuture>, CommandFactoryContext) -> (Box<dyn GpuFuture>, Arc<ImageView<Arc<AttachmentImage>>>)
    {
        let image = image_create(&self.info);
        let (mut buffers, mut fut_local) = self.create_buffers(image.clone(), scene);
        let fut: Box<dyn GpuFuture> = Box::new(previous.join(fut_local));

        let mut ctx = CommandFactoryContext {
            app_info: &self.info,
            buffers: buffers.clone(),
            scene,
            camera: &scene.data.camera,
            render_step: 0
        };

        let mut fut = {
            let cmd = ctx.create_command_buffer()
                .update_with(|buf| {
                    buf.0.clear_color_image(ctx.buffers.image.image().clone(), ClearValue::Float([0.0; 4])).unwrap();
                })
                .build();
            fut.then_execute(ctx.graphics_queue(), cmd)
                .unwrap()
                .boxed()
        };

        for i in 0..self.info.render_steps {
            let ctx = CommandFactoryContext {
                app_info: &self.info,
                buffers: buffers.clone(),
                scene,
                camera: &scene.data.camera,
                render_step: i
            };
            for factory in self.commands.iter_mut() {
                fut = factory.make_command(ctx.clone(), fut);
            }
            let (buffers_, fut_local) = self.create_buffers(image.clone(), scene);
            buffers = buffers_;
            fut = fut.join(fut_local).boxed();
        }

        let ctx = CommandFactoryContext {
            app_info: &self.info,
            buffers: buffers.clone(),
            scene,
            camera: &scene.data.camera,
            render_step: 0
        };

        Ok(add_msaa(fut, ctx))
    }
    fn create_buffers(
        &mut self,
        image: Arc<ImageView<Arc<AttachmentImage>>>,
        scene: &mut Scene,
    ) -> (Buffers, Box<dyn GpuFuture>) {
        self.buffers.make_buffers(&self.info, image, scene)
    }
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

    pub fn make_buffers(
        &mut self,
        app: &AppInfo,
        image: Arc<ImageView<Arc<AttachmentImage>>>,
        scene: &mut Scene,
    ) -> (Buffers, Box<dyn GpuFuture>) {
        let (model_bufs, fut) = scene.frame_buffers(app);
        let bufs = self.sets.buffers(
            self.intersections.clone(),
            Arc::new(
                self.intersections_count
                    .chunk(std::iter::once(DispatchIndirectCommand { x: 0, y: 0, z: 0 }))
                    .unwrap(),
            ),
            Arc::new(
                self.camera.next(scene.data.camera.clone().into_uniform().as_std140()).unwrap(),
            ),
            Arc::new(self.screen.next(app.screen.clone()).unwrap()),
            image,
            Arc::new(
                self.direction_light.next(scene.data.global_light.clone().into_uniform()).unwrap(),
            ),
            model_bufs,
        );
        (bufs, fut)
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
    pub intersections_set: FixedSizeDescriptorSetsPool,
    pub models_set: FixedSizeDescriptorSetsPool,
    pub sphere_models_set: FixedSizeDescriptorSetsPool,
    pub lights_set: FixedSizeDescriptorSetsPool,
    pub image_set: FixedSizeDescriptorSetsPool,
    pub workgroups_set: FixedSizeDescriptorSetsPool,
}

impl SetsStorage {
    pub fn new(device: &Arc<Device>) -> Self {
        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                path: "shaders/sets.glsl",
                include: ["OPTIMIZE_NO"]
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
                        &(),
                        None,
                    )
                    .unwrap(),
                )
            })
            .clone();

        let _ = 2;
        let global_app_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(0).unwrap().clone(),
        );
        let intersections_set = FixedSizeDescriptorSetsPool::new(
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
        let workgroups_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(6).unwrap().clone(),
        );

        SetsStorage {
            global_app_set,
            intersections_set,
            models_set,
            sphere_models_set,
            lights_set,
            image_set,
            workgroups_set,
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

        let models_set = models_buffers.models_set.clone();

        let sphere_models_set = models_buffers.sphere_models_set.clone();

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

        let intersections_set = Arc::new(
            self.intersections_set
                .next()
                .add_buffer(intersections.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let image_set = Arc::new(
            self.image_set.next().add_image(output_image.clone()).unwrap().build().unwrap(),
        );
        let workgroups_set = Arc::new(
            self.workgroups_set
                .next()
                .add_buffer(intersections_count.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        Buffers {
            sphere_models_set,
            image: output_image,
            workgroups: intersections_count,
            intersections,
            global_app_set,
            models_set,
            lights_set,
            intersections_set,
            image_set,
            workgroups_set,
        }
    }
}

#[derive(Clone)]
pub struct Buffers {
    pub intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    pub workgroups: Arc<CpuBufferPoolChunk<DispatchIndirectCommand, Arc<StdMemoryPool>>>,
    pub image: Arc<ImageView<Arc<AttachmentImage>>>,

    pub global_app_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub intersections_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub models_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub sphere_models_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub lights_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub image_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub workgroups_set: Arc<dyn DescriptorSet + Send + Sync>,
}

pub type Rays = [Ray];

pub struct AppBuilder {
    info: AppInfo,
    commands: Vec<Box<dyn CommandFactory>>,
    global_buffers: GlobalBuffers,
}

impl AppBuilder {
    pub fn info(&self) -> &AppInfo {
        &self.info
    }
    pub fn commands(&self) -> &Vec<Box<dyn CommandFactory>> {
        &self.commands
    }
}

impl AppBuilder {
    pub fn new(info: AppInfo) -> Self {
        let buffers = GlobalBuffers::new(
            &info.device,
            info.graphics_queue.family(),
            info.size_of_image_array(),
        );
        Self { info, commands: vec![], global_buffers: buffers }
    }
    pub fn then_command(mut self, f: Box<dyn CommandFactory>) -> Self {
        self.commands.push(f);
        self
    }
    pub fn build(self) -> App {
        App::new(self.info, self.commands, self.global_buffers)
    }
}
