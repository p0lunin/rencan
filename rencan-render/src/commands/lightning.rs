use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder},
    descriptor::{
        descriptor_set::PersistentDescriptorSet, pipeline_layout::PipelineLayout,
        PipelineLayoutAbstract,
    },
    device::Device,
    pipeline::ComputePipeline,
};

use crate::{
    commands::shaders::ray_trace_shader,
    core::{
        intersection::IntersectionUniform, BufferAccessData, CommandFactory, CommandFactoryContext,
        Ray,
    },
};
use vulkano::buffer::{DeviceLocalBuffer, BufferUsage, TypedBufferAccess};
use crate::core::app::GlobalAppBuffers;
use std::cell::{Cell, RefCell};

mod lightning_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/lightning.glsl"
    }
}

mod make_shadow_rays_cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "shaders/make_shadow_rays.glsl"
    }
}

pub struct LightningCommandFactory {
    device: Arc<Device>,
    ray_trace_pipeline: Arc<ComputePipeline<PipelineLayout<ray_trace_shader::Layout>>>,
    make_shadow_rays_pipeline: Arc<ComputePipeline<PipelineLayout<make_shadow_rays_cs::Layout>>>,
    lightning_pipeline: Arc<ComputePipeline<PipelineLayout<lightning_cs::Layout>>>,
    global_buffers: GlobalAppBuffers,
    lights_count: Cell<u32>,
    local_ray_buffer: RefCell<Arc<DeviceLocalBuffer<[Ray]>>>,
    local_intersections_buffer: RefCell<Arc<DeviceLocalBuffer<[IntersectionUniform]>>>,
}

impl LightningCommandFactory {
    pub fn new(buffers: GlobalAppBuffers, device: Arc<Device>) -> Self {
        let ray_trace_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &ray_trace_shader::Shader::load(device.clone()).unwrap().main_entry_point(),
                &(),
                None,
            )
            .unwrap(),
        );
        let make_shadow_rays_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &make_shadow_rays_cs::Shader::load(device.clone()).unwrap().main_entry_point(),
                &(),
                None,
            )
            .unwrap(),
        );
        let lightning_pipeline = Arc::new(
            ComputePipeline::new(
                device.clone(),
                &lightning_cs::Shader::load(device.clone()).unwrap().main_entry_point(),
                &(),
                None,
            )
            .unwrap(),
        );
        let local_ray_buffer = RefCell::new(DeviceLocalBuffer::array(
            device.clone(),
            buffers.rays.len(),
            BufferUsage {
                storage_buffer: true,
                transfer_destination: true,
                ..BufferUsage::none()
            },
            buffers.rays.queue_families()
        ).unwrap());
        let local_intersections_buffer = RefCell::new(DeviceLocalBuffer::array(
            device.clone(),
            buffers.intersections.len(),
            BufferUsage {
                storage_buffer: true,
                transfer_destination: true,
                ..BufferUsage::none()
            },
            buffers.intersections.queue_families()
        ).unwrap());
        LightningCommandFactory {
            device,
            ray_trace_pipeline,
            make_shadow_rays_pipeline,
            lightning_pipeline,
            global_buffers: buffers,
            local_ray_buffer,
            local_intersections_buffer,
            lights_count: Cell::new(1),
        }
    }
}

impl CommandFactory for LightningCommandFactory {
    fn make_command(&self, ctx: CommandFactoryContext) -> AutoCommandBuffer {
        let light_counts = ctx.scene.point_lights.len() as u32;
        if self.lights_count.get() != light_counts + 1 {
            *self.local_ray_buffer.borrow_mut() = DeviceLocalBuffer::array(
                self.device.clone(),
                ctx.app_info.size_of_image_array() * (1 + light_counts as usize),
                BufferUsage::all(),
                self.global_buffers.rays.queue_families(),
            ).unwrap();
            *self.local_intersections_buffer.borrow_mut() = DeviceLocalBuffer::array(
                self.device.clone(),
                ctx.app_info.size_of_image_array() * (1 + light_counts as usize),
                BufferUsage::all(),
                self.global_buffers.intersections.queue_families(),
            ).unwrap();
            self.lights_count.set(light_counts + 1);
        }

        let mut command = AutoCommandBufferBuilder::new(
            ctx.app_info.device.clone(),
            ctx.app_info.graphics_queue.family(),
        )
        .unwrap();

        command.fill_buffer(self.local_intersections_buffer.borrow().clone(), 0).unwrap();
        command.fill_buffer(self.local_ray_buffer.borrow().clone(), 0).unwrap();

        let rays = add_making_shadow_rays(self, &self.global_buffers, &ctx, self.local_ray_buffer.borrow().clone(), &mut command);
        let intersections = add_ray_tracing(self, light_counts, &ctx, rays.clone(), self.local_intersections_buffer.borrow().clone(), &mut command);
        add_lightning(self, &self.global_buffers, &ctx, rays, intersections, &mut command);

        let command = command.build().unwrap();

        command
    }

    fn update_buffers(&mut self, buffers: GlobalAppBuffers) {
        self.global_buffers = buffers;
        self.lights_count.set(0);
    }
}

fn add_lightning(
    factory: &LightningCommandFactory,
    global_buffers: &GlobalAppBuffers,
    ctx: &CommandFactoryContext,
    rays: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    intersections: Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync>,
    command: &mut AutoCommandBufferBuilder,
) {
    let CommandFactoryContext { buffers, count_of_workgroups, .. } = ctx;

    let layout_0 = factory.lightning_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(global_buffers.rays.clone())
            .unwrap()
            .add_buffer(rays.clone())
            .unwrap()
            .add_image(buffers.output_image.clone())
            .unwrap()
            .add_buffer(global_buffers.intersections.clone())
            .unwrap()
            .add_buffer(intersections.clone())
            .unwrap()
            .add_buffer(buffers.direction_light.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.point_lights_count.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.point_lights.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let layout_1 = factory.lightning_pipeline.layout().descriptor_set_layout(1).unwrap();
    let set_1 = Arc::new(
        PersistentDescriptorSet::start(layout_1.clone())
            .add_buffer(buffers.models_buffers.infos.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.vertices.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.indices.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    command
        .dispatch(
            [*count_of_workgroups, 1, 1],
            factory.lightning_pipeline.clone(),
            (set_0, set_1),
            (),
        )
        .unwrap();
}

fn add_making_shadow_rays(
    factory: &LightningCommandFactory,
    global_buffers: &GlobalAppBuffers,
    ctx: &CommandFactoryContext,
    new_rays_buffer: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    command: &mut AutoCommandBufferBuilder,
) -> Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync> {
    let CommandFactoryContext { buffers, count_of_workgroups, .. } = ctx;

    let layout_0 = factory.make_shadow_rays_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(global_buffers.rays.clone())
            .unwrap()
            .add_buffer(global_buffers.intersections.clone())
            .unwrap()
            .add_buffer(new_rays_buffer.clone())
            .unwrap()
            .add_buffer(buffers.direction_light.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.point_lights_count.clone())
            .unwrap()
            .add_buffer(buffers.models_buffers.point_lights.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    command
        .dispatch(
            [*count_of_workgroups, 1, 1],
            factory.make_shadow_rays_pipeline.clone(),
            set_0,
            (),
        )
        .unwrap();

    new_rays_buffer
}

fn add_ray_tracing(
    factory: &LightningCommandFactory,
    lights_count: u32,
    ctx: &CommandFactoryContext,
    rays: Arc<dyn BufferAccessData<Data = [Ray]> + Send + Sync>,
    intersections: Arc<DeviceLocalBuffer<[IntersectionUniform]>>,
    command: &mut AutoCommandBufferBuilder,
) -> Arc<dyn BufferAccessData<Data = [IntersectionUniform]> + Send + Sync> {
    let CommandFactoryContext { buffers, count_of_workgroups, .. } = ctx;

    let layout_0 = factory.ray_trace_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set_0 = Arc::new(
        PersistentDescriptorSet::start(layout_0.clone())
            .add_buffer(buffers.screen.clone())
            .unwrap()
            .add_buffer(rays.clone())
            .unwrap()
            .add_buffer(intersections.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let layout_1 = factory.ray_trace_pipeline.layout().descriptor_set_layout(1).unwrap();
     let set_1 = Arc::new(
            PersistentDescriptorSet::start(layout_1.clone())
                .add_buffer(buffers.models_buffers.count.clone())
                .unwrap()
                .add_buffer(buffers.models_buffers.infos.clone())
                .unwrap()
                .add_buffer(buffers.models_buffers.vertices.clone())
                .unwrap()
                .add_buffer(buffers.models_buffers.indices.clone())
                .unwrap()
                .add_buffer(buffers.models_buffers.hit_boxes.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

    command
        .dispatch(
            [*count_of_workgroups * (1 + lights_count), 1, 1],
            factory.ray_trace_pipeline.clone(),
            (set_0, set_1),
            (),
        )
        .unwrap();

    intersections
}
