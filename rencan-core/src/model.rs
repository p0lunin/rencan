use crevice::std140::AsStd140;
use nalgebra::{Point3, Point4, Similarity3, Translation3, UnitQuaternion, Isometry3};
use std::sync::Arc;
use vulkano::buffer::{CpuBufferPool, BufferUsage, ImmutableBuffer};
use vulkano::device::{Device, Queue};
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::memory::pool::StdMemoryPool;
use crate::BufferAccessData;
use vulkano::sync::GpuFuture;
use once_cell::sync::OnceCell;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Point4<f32>>,
    pub indexes: Vec<Point4<u32>>,
    pub rotation: UnitQuaternion<f32>,
    pub position: Point3<f32>,
    pub scaling: f32,
    pub albedo: f32,
}

impl Model {
    pub fn new(vertices: Vec<Point4<f32>>, indexes: Vec<Point4<u32>>) -> Self {
        Model {
            vertices,
            indexes,
            rotation: UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, 0.0),
            scaling: 1.0,
            albedo: 0.18,
        }
    }
    pub fn with_isometry(
        vertices: Vec<Point4<f32>>,
        indexes: Vec<Point4<u32>>,
        rotation: UnitQuaternion<f32>,
        position: Point3<f32>,
        scaling: f32,
        albedo: f32,
    ) -> Self {
        Model { vertices, indexes, rotation, position, scaling, albedo }
    }
    pub fn get_uniform_info(&self, model_id: u32) -> ModelUniformInfo {
        ModelUniformInfo {
            isometry: (Isometry3::from_parts(
                Translation3::new(self.position.x, self.position.y, self.position.z),
                self.rotation,
            )
            .to_matrix() * self.scaling)
            .into(),
            model_id,
            indexes_length: self.indexes.len() as u32,
            albedo: self.albedo,
        }
    }
}

#[derive(AsStd140)]
pub struct ModelUniformInfo {
    pub isometry: mint::ColumnMatrix4<f32>,
    pub model_id: u32,
    pub indexes_length: u32,
    pub albedo: f32,
}

impl ModelUniformInfo {
    pub fn as_std140(&self) -> <Self as AsStd140>::Std140Type {
        AsStd140::as_std140(self)
    }
}

type ModelUniformInfoStd140 = <ModelUniformInfo as AsStd140>::Std140Type;

#[allow(dead_code)]
pub struct AppModel {
    model: Model,
    buffer_info: OnceCell<Arc<CpuBufferPool<ModelUniformInfoStd140>>>,
    buffer_vertices: OnceCell<Arc<ImmutableBuffer<[Point4<f32>]>>>,
    buffer_indices: OnceCell<Arc<ImmutableBuffer<[Point4<u32>]>>>,
    need_update_info: RefCell<bool>,
    need_update_vertices: bool,
    need_update_indices: bool,
    info_chunk: RefCell<Option<Arc<CpuBufferPoolSubbuffer<ModelUniformInfoStd140, Arc<StdMemoryPool>>>>>
}

impl AppModel {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            buffer_info: OnceCell::new(),
            buffer_vertices: OnceCell::new(),
            buffer_indices: OnceCell::new(),
            need_update_info: RefCell::new(true),
            need_update_vertices: true,
            need_update_indices: true,
            info_chunk: RefCell::new(None),
        }
    }
    pub fn model(&self) -> &Model {
        &self.model
    }
    pub fn update_info(&mut self, f: impl FnOnce(&mut Model)) {
        f(&mut self.model);
        *self.need_update_info.borrow_mut() = true;
    }
    pub fn get_info_buffer(&self, device: &Arc<Device>, id: u32) -> Arc<dyn BufferAccessData<Data = ModelUniformInfoStd140> + Send + Sync> {
        let buf = match self.buffer_info.get() {
            Some(buf) =>{
                buf
            }
            None => {
                self.buffer_info.set(Arc::new(CpuBufferPool::new(
                    device.clone(),
                    BufferUsage::all(),
                ))).unwrap_or_else(|_| panic!("We already checks this"));
                self.buffer_info.get().unwrap()
            }
        };
        if *self.need_update_info.borrow() {
            let chunk = Arc::new(buf.next(self.model.get_uniform_info(id).as_std140()).unwrap());
            *self.info_chunk.borrow_mut() = Some(chunk.clone());
            *self.need_update_info.borrow_mut() = false;
            chunk
        }
        else {
            self.info_chunk.borrow().as_ref().unwrap().clone()
        }
    }
    pub fn get_vertices_buffer(&self, queue: &Arc<Queue>) -> Arc<dyn BufferAccessData<Data = [Point4<f32>]>  + Send + Sync> {
        match self.buffer_vertices.get() {
            Some(buf) => {
                buf.clone()
            }
            None => {
                let (buf, fut) = ImmutableBuffer::from_iter(
                    self.model.vertices.iter().cloned(),
                    BufferUsage::all(),
                    queue.clone()
                ).unwrap();
                fut.then_signal_fence().wait(None).unwrap();
                self.buffer_vertices.set(buf.clone()).unwrap_or_else(|_| panic!("We already check it above"));
                buf
            }
        }
    }
    pub fn get_indices_buffer(&self, queue: &Arc<Queue>) -> Arc<dyn BufferAccessData<Data = [Point4<u32>]> + Send + Sync> {
        match self.buffer_indices.get() {
            Some(buf) => {
                buf.clone()
            }
            None => {
                let (buf, fut) = ImmutableBuffer::from_iter(
                    self.model.indexes.iter().cloned(),
                    BufferUsage::all(),
                    queue.clone()
                ).unwrap();
                fut.then_signal_fence().wait(None).unwrap();
                self.buffer_indices.set(buf.clone()).unwrap_or_else(|_| panic!("We already check it above"));
                buf
            }
        }
    }
}
