use vulkano::{
    buffer::{
        cpu_pool::CpuBufferPoolSubbuffer, BufferAccess, CpuAccessibleBuffer, DeviceLocalBuffer,
        ImmutableBuffer,
    },
    memory::MemoryPool,
};

pub trait BufferAccessData: BufferAccess {
    type Data: ?Sized;
}

impl<T: ?Sized + Sync + Send + 'static, A> BufferAccessData for CpuAccessibleBuffer<T, A> {
    type Data = T;
}

impl<T: ?Sized + Sync + Send + 'static, A> BufferAccessData for DeviceLocalBuffer<T, A> {
    type Data = T;
}

impl<T: ?Sized + Sync + Send + 'static, A> BufferAccessData for ImmutableBuffer<T, A> {
    type Data = T;
}

impl<T: Sync + Send + 'static, A: MemoryPool> BufferAccessData for CpuBufferPoolSubbuffer<T, A> {
    type Data = T;
}
