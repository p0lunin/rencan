use crate::{hitbox::HitBoxRectangleUniformStd140, light::PointLightUniform, model::ModelUniformInfo, Scene, AppInfo};
use crevice::std140::AsStd140;
use nalgebra::Point4;
use std::sync::Arc;
use vulkano::{
    buffer::{
        cpu_pool::{CpuBufferPoolChunk, CpuBufferPoolSubbuffer},
        BufferUsage, CpuBufferPool, TypedBufferAccess,
    },
    device::Device,
    memory::pool::StdMemoryPool,
};
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use once_cell::sync::OnceCell;
use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::pipeline::ComputePipeline;
use vulkano::descriptor::{PipelineLayoutAbstract, DescriptorSet};
use crate::scene::SceneData;
use vulkano::buffer::ImmutableBuffer;
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;

pub struct SceneBuffersStorage {
    pub counts_u32: CpuBufferPool<u32>,
    pub model_infos: CpuBufferPool<ModelUniformInfo>,
    pub vertices: CpuBufferPool<Point4<f32>>,
    pub indices: CpuBufferPool<Point4<u32>>,
    pub hit_boxes: CpuBufferPool<HitBoxRectangleUniformStd140>,
    pub point_lights: CpuBufferPool<PointLightUniform>,
    pub point_lights_count: CpuBufferPool<u32>,

    pub sphere_count: CpuBufferPool<u32, Arc<StdMemoryPool>>,
    pub sphere_infos: CpuBufferPool<ModelUniformInfo, Arc<StdMemoryPool>>,
    pub spheres: CpuBufferPool<Point4<f32>, Arc<StdMemoryPool>>,

    pub models_set: FixedSizeDescriptorSetsPool,
    pub sphere_models_set: FixedSizeDescriptorSetsPool,
}

impl SceneBuffersStorage {
    pub fn init(device: Arc<Device>) -> Self {
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
            .get_or_init({
                let device = device.clone();
                move || {
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
                }
            })
            .clone();

        let models_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(2).unwrap().clone(),
        );
        let sphere_models_set = FixedSizeDescriptorSetsPool::new(
            pip.layout().descriptor_set_layout(3).unwrap().clone(),
        );

        Self {
            counts_u32: CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer()),
            model_infos: CpuBufferPool::new(
                device.clone(),
                BufferUsage::transfer_source(),
            ),
            vertices: CpuBufferPool::new(
                device.clone(),
                BufferUsage::transfer_source(),
            ),
            indices: CpuBufferPool::new(
                device.clone(),
                BufferUsage::transfer_source(),
            ),
            hit_boxes: CpuBufferPool::new(
                device.clone(),
                BufferUsage::transfer_source(),
            ),
            point_lights: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            point_lights_count: CpuBufferPool::new(
                device.clone(),
                BufferUsage { uniform_buffer: true, ..BufferUsage::none() },
            ),
            sphere_count: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            sphere_infos: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            spheres: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            models_set,
            sphere_models_set,
        }
    }

    pub fn get_buffers(&mut self, app: &AppInfo, scene: &mut SceneData) -> (SceneBuffers, Box<dyn GpuFuture>) {
        let mut fut: Box<dyn GpuFuture> = Box::new(vulkano::sync::now(app.device.clone()));
        let point_lights = &scene.point_lights;

        let models_set = scene.models.get_depends_or_init(|models| {
            let count = self.counts_u32.next(models.len() as u32).unwrap();
            let infos = self
                .model_infos
                .chunk(models.iter().enumerate().map(|(i, m)| m.model().get_uniform_info(i as u32)))
                .unwrap();
            let vertices = self
                .vertices
                .chunk(
                    models
                        .iter()
                        .map(|m| m.model().vertices.iter().cloned())
                        .flatten()
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
                .unwrap();
            let indices = self
                .indices
                .chunk(
                    models
                        .iter()
                        .map(|m| m.model().indexes.iter().cloned())
                        .flatten()
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
                .unwrap();
            let hit_boxes = self
                .hit_boxes
                .chunk(models.iter().map(|m| m.hit_box().clone().into_uniform().as_std140()))
                .unwrap();

            let (infos, fu2) = ImmutableBuffer::from_buffer(
                infos,
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
                app.graphics_queue.clone()
            ).unwrap();
            let (vertices, fu3) = ImmutableBuffer::from_buffer(
                vertices,
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
                app.graphics_queue.clone()
            ).unwrap();
            let (indices, fu4) = ImmutableBuffer::from_buffer(
                indices,
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
                app.graphics_queue.clone()
            ).unwrap();
            let (hit_boxes, fu5) = ImmutableBuffer::from_buffer(
                hit_boxes,
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
                app.graphics_queue.clone()
            ).unwrap();

            fut = Box::new(fu2.join(fu3).join(fu4).join(fu5));

            let models_set = Arc::new(
                self.models_set
                    .next()
                    .add_buffer(count.clone())
                    .unwrap()
                    .add_buffer(infos.clone())
                    .unwrap()
                    .add_buffer(vertices.clone())
                    .unwrap()
                    .add_buffer(indices.clone())
                    .unwrap()
                    .add_buffer(hit_boxes.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            );

            models_set
        }).clone();

        let point_lights =
            self.point_lights.chunk(point_lights.iter().map(|l| l.clone().into_uniform())).unwrap();
        let point_lights_count = self.point_lights_count.next(point_lights.len() as u32).unwrap();

        let sphere_models_set = scene.sphere_models.get_depends_or_init(|sphere_models| {
            let sphere_count = self.sphere_count.next(sphere_models.len() as u32).unwrap();
            let sphere_infos = self
                .sphere_infos
                .chunk(
                    sphere_models.iter().enumerate().map(|(i, m)| m.get_uniform_info(i as u32)),
                )
                .unwrap();
            let spheres = self
                .spheres
                .chunk(
                    sphere_models
                        .iter()
                        .map(|m| Point4::new(m.center.x, m.center.y, m.center.z, m.radius))
                        .collect::<Vec<_>>()
                        .into_iter(),
                )
                .unwrap();

            let sphere_models_set = Arc::new(
                self.sphere_models_set
                    .next()
                    .add_buffer(sphere_count.clone())
                    .unwrap()
                    .add_buffer(sphere_infos.clone())
                    .unwrap()
                    .add_buffer(spheres.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            );
            sphere_models_set
        }).clone();

        let bufs = SceneBuffers {
            point_lights_count,
            point_lights,
            models_set,
            sphere_models_set,
        };

        (bufs, fut)
    }
}

#[derive(Clone)]
pub struct SceneBuffers {
    pub point_lights_count: CpuBufferPoolSubbuffer<u32, Arc<StdMemoryPool>>,
    pub point_lights: CpuBufferPoolChunk<PointLightUniform, Arc<StdMemoryPool>>,

    pub models_set: Arc<dyn DescriptorSet + Send + Sync>,
    pub sphere_models_set: Arc<dyn DescriptorSet + Send + Sync>,
}
