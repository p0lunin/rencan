use nalgebra::{Point3, Point4};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBuffer},
    device::{Device, DeviceExtensions, Features, Queue},
    image::{AttachmentImage, ImageAccess, ImageUsage},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
    sync::GpuFuture,
};

pub fn init_vk_instance() -> Arc<Instance> {
    Instance::new(None, &InstanceExtensions::none(), None).unwrap()
}

pub fn pick_device_and_queue(instance: &Arc<Instance>) -> (Arc<Device>, Arc<Queue>) {
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let queue_family = physical.queue_families().find(|&q| q.supports_graphics()).unwrap();

    let (device, mut queues) = Device::new(
        physical,
        &Features::none(),
        &DeviceExtensions { khr_storage_buffer_storage_class: true, ..DeviceExtensions::none() },
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    (device, queue)
}

pub fn to_buffer<T>(device: Arc<Device>, data: T) -> Arc<CpuAccessibleBuffer<T>>
where
    T: 'static,
{
    CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, data).unwrap()
}

pub fn run_one(command: AutoCommandBuffer, queue: Arc<Queue>) {
    command.execute(queue).unwrap().then_signal_fence_and_flush().unwrap().wait(None).unwrap();
}

pub fn empty_array<T: 'static>(device: Arc<Device>, size: usize, make: impl Fn() -> T) -> Arc<CpuAccessibleBuffer<[T]>> {
    CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..size).map(|_| make()),
    )
    .unwrap()
}

pub fn empty_image(device: Arc<Device>) -> Arc<AttachmentImage> {
    AttachmentImage::with_usage(
        device.clone(),
        [1, 1],
        vulkano::format::Format::R8G8B8A8Uint,
        ImageUsage { storage: true, ..ImageUsage::none() },
    )
    .unwrap()
}

pub fn rays_from_vec(
    device: Arc<Device>,
    vec: Vec<Point4<f32>>,
) -> Arc<CpuAccessibleBuffer<[Point4<f32>]>> {
    CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vec.into_iter())
        .unwrap()
}

pub fn copy_image_to_buf(
    device: Arc<Device>,
    queue: Arc<Queue>,
    image: Arc<dyn ImageAccess + Send + Sync + 'static>,
    buffer: Arc<CpuAccessibleBuffer<[[u8; 4]]>>,
) -> AutoCommandBuffer {
    let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    builder.copy_image_to_buffer(image, buffer).unwrap();
    builder.build().unwrap()
}
