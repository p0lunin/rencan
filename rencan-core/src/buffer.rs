use vulkano::buffer::{BufferAccess, CpuAccessibleBuffer, DeviceLocalBuffer};

pub trait BufferAccessData: BufferAccess {
    type Data: ?Sized;
}

impl<T: ?Sized + Sync + Send + 'static, A> BufferAccessData for CpuAccessibleBuffer<T, A> {
    type Data = T;
}

impl<T: ?Sized + Sync + Send + 'static, A> BufferAccessData for DeviceLocalBuffer<T, A> {
    type Data = T;
}
