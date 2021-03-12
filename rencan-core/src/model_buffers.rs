use crate::{
    hitbox::HitBoxRectangleUniformStd140, light::PointLightUniform, model::ModelUniformInfo, Scene,
};
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

type ModelUniformInfoStd140 = <ModelUniformInfo as AsStd140>::Std140Type;

pub struct SceneBuffersStorage {
    pub counts_u32: CpuBufferPool<u32>,
    pub model_infos: CpuBufferPool<ModelUniformInfoStd140>,
    pub vertices: CpuBufferPool<Point4<f32>>,
    pub indices: CpuBufferPool<Point4<u32>>,
    pub hit_boxes: CpuBufferPool<HitBoxRectangleUniformStd140>,
    pub point_lights: CpuBufferPool<PointLightUniform>,
    pub point_lights_count: CpuBufferPool<u32>,
}

impl SceneBuffersStorage {
    pub fn init(device: Arc<Device>) -> Self {
        Self {
            counts_u32: CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer()),
            model_infos: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            vertices: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            indices: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            hit_boxes: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            point_lights: CpuBufferPool::new(
                device.clone(),
                BufferUsage { storage_buffer: true, ..BufferUsage::none() },
            ),
            point_lights_count: CpuBufferPool::new(
                device.clone(),
                BufferUsage { uniform_buffer: true, ..BufferUsage::none() },
            ),
        }
    }

    pub fn get_buffers(&self, scene: &Scene) -> SceneBuffers {
        let models = &scene.models;
        let point_lights = &scene.point_lights;

        let count = self.counts_u32.next(models.len() as u32).unwrap();
        let infos = self
            .model_infos
            .chunk(
                models
                    .iter()
                    .enumerate()
                    .map(|(i, m)| m.model().get_uniform_info(i as u32).as_std140()),
            )
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
        let point_lights =
            self.point_lights.chunk(point_lights.iter().map(|l| l.clone().into_uniform())).unwrap();
        let point_lights_count = self.point_lights_count.next(point_lights.len() as u32).unwrap();
        SceneBuffers {
            count,
            infos,
            vertices,
            indices,
            hit_boxes,
            point_lights_count,
            point_lights,
        }
    }
}

#[derive(Clone)]
pub struct SceneBuffers {
    pub count: CpuBufferPoolSubbuffer<u32, Arc<StdMemoryPool>>,
    pub infos: CpuBufferPoolChunk<ModelUniformInfoStd140, Arc<StdMemoryPool>>,
    pub vertices: CpuBufferPoolChunk<Point4<f32>, Arc<StdMemoryPool>>,
    pub indices: CpuBufferPoolChunk<Point4<u32>, Arc<StdMemoryPool>>,
    pub hit_boxes: CpuBufferPoolChunk<HitBoxRectangleUniformStd140, Arc<StdMemoryPool>>,
    pub point_lights_count: CpuBufferPoolSubbuffer<u32, Arc<StdMemoryPool>>,
    pub point_lights: CpuBufferPoolChunk<PointLightUniform, Arc<StdMemoryPool>>,
}
