use crate::{
    camera::{Camera, CameraUniform},
    commands,
};
use crevice::std140::AsStd140;
use nalgebra::Point4;
use rencan_core::{AppInfo, BufferAccessData, Screen};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    command_buffer::{AutoCommandBuffer, CommandBufferExecFuture},
    image::ImageViewAccess,
    sync::{GpuFuture},
};

pub struct App {
    info: AppInfo,
    camera: Camera,
}

type RenderFut<F> =
    CommandBufferExecFuture<CommandBufferExecFuture<F, AutoCommandBuffer>, AutoCommandBuffer>;

impl App {
    pub fn new(info: AppInfo, camera: Camera) -> Self {
        Self { info, camera }
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
        image_create: F,
    ) -> (RenderFut<Prev>, Arc<dyn ImageViewAccess + Send + Sync + 'static>)
    where
        Prev: GpuFuture,
        F: FnOnce(&AppInfo) -> Arc<dyn ImageViewAccess + Send + Sync + 'static>,
    {
        let image = image_create(&self.info);
        let buffers = self.create_buffers(image.clone());
        let compute_rays = commands::compute_rays(
            &self.info,
            buffers.screen.clone(),
            buffers.camera.clone(),
            buffers.rays.clone(),
        );
        let show_ordinates = commands::show_xyz_ordinates(
            &self.info,
            self.camera.position().clone(),
            buffers.output_image.clone(),
            buffers.rays.clone(),
            buffers.screen.clone(),
        );
        let fut = previous
            .then_execute(self.info.graphics_queue.clone(), compute_rays)
            .unwrap()
            .then_execute_same_queue(show_ordinates)
            .unwrap();

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
