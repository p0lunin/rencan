use vulkano::buffer::{CpuBufferPool, BufferUsage};
use std::sync::Arc;
use nalgebra::Point4;
use crate::model::{AppModel, ModelUniformInfo};
use crevice::std140::AsStd140;
use vulkano::device::Device;
use vulkano::buffer::cpu_pool::{CpuBufferPoolSubbuffer, CpuBufferPoolChunk};
use vulkano::memory::pool::StdMemoryPool;
use crate::hitbox::HitBoxRectangleUniformStd140;

type ModelUniformInfoStd140 = <ModelUniformInfo as AsStd140>::Std140Type;

pub struct ModelsBuffersStorage {
    pub model_counts: CpuBufferPool<u32>,
    pub model_infos: CpuBufferPool<ModelUniformInfoStd140>,
    pub vertices: CpuBufferPool<Point4<f32>>,
    pub indices: CpuBufferPool<Point4<u32>>,
    pub hit_boxes: CpuBufferPool<HitBoxRectangleUniformStd140>,
}

impl ModelsBuffersStorage {
    pub fn init(device: Arc<Device>) -> Self {
        Self {
            model_counts: CpuBufferPool::new(
                device.clone(),
                BufferUsage::uniform_buffer(),
            ),
            model_infos: CpuBufferPool::new(
                device.clone(),
                BufferUsage {
                    storage_buffer:true,
                    ..BufferUsage::none()
                },
            ),
            vertices: CpuBufferPool::new(
                device.clone(),
                BufferUsage {
                    storage_buffer:true,
                    ..BufferUsage::none()
                },
            ),
            indices: CpuBufferPool::new(
                device.clone(),
                BufferUsage {
                    storage_buffer:true,
                    ..BufferUsage::none()
                },
            ),
            hit_boxes: CpuBufferPool::new(
                device.clone(),
                BufferUsage {
                    storage_buffer:true,
                    ..BufferUsage::none()
                },
            ),
        }
    }

    pub fn get_buffers(&self, models: &[AppModel]) -> ModelsBuffers {
        let count = self.model_counts.next(models.len() as u32).unwrap();
        let infos = self.model_infos.chunk(
            models
                .iter()
                .enumerate()
                .map(|(i, m)|
                    m.model().get_uniform_info(i as u32).as_std140()
                )
        ).unwrap();
        let vertices = self.vertices.chunk(
            models
                .iter()
                .map(|m|
                    m.model().vertices.iter().cloned()
                )
                .flatten()
                .collect::<Vec<_>>()
                .into_iter()
        ).unwrap();
        let indices = self.indices.chunk(
            models
                .iter()
                .map(|m|
                    m.model().indexes.iter().cloned()
                )
                .flatten()
                .collect::<Vec<_>>()
                .into_iter()
        ).unwrap();
        let hit_boxes = self.hit_boxes.chunk(
            models
                .iter()
                .map(|m|
                    m.hit_box().clone().into_uniform().as_std140()
                )
        ).unwrap();
        ModelsBuffers {
            count,
            infos,
            vertices,
            indices,
            hit_boxes,
        }
    }
}

#[derive(Clone)]
pub struct ModelsBuffers {
    pub count: CpuBufferPoolSubbuffer<u32, Arc<StdMemoryPool>>,
    pub infos: CpuBufferPoolChunk<ModelUniformInfoStd140, Arc<StdMemoryPool>>,
    pub vertices: CpuBufferPoolChunk<Point4<f32>, Arc<StdMemoryPool>>,
    pub indices: CpuBufferPoolChunk<Point4<u32>, Arc<StdMemoryPool>>,
    pub hit_boxes: CpuBufferPoolChunk<HitBoxRectangleUniformStd140, Arc<StdMemoryPool>>,
}
