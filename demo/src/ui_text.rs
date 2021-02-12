use std::sync::Arc;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState, SubpassContents,
};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::instance::QueueFamily;
use vulkano::pipeline::vertex::{BufferlessDefinition, BufferlessVertices};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::{single_pass_renderpass, impl_vertex};
use winit::dpi::{LogicalSize, Size};
use winit::{event_loop::EventLoop, window::WindowBuilder};
use vulkano::buffer::{BufferAccess, CpuAccessibleBuffer, BufferUsage};

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_LUNARG_standard_validation"];

fn create_render_pass(
    device: &Arc<Device>,
    color_format: Format,
) -> Arc<dyn RenderPassAbstract + Send + Sync> {
    Arc::new(
        single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    )
}

fn create_graphics_pipeline(
    device: &Arc<Device>,
    swap_chain_extent: [u32; 2],
    render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
    mod vertex_shader {
        vulkano_shaders::shader! {
           ty: "vertex",
           path: "shaders/vertex.glsl"
        }
    }

    mod fragment_shader {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "shaders/fragment.glsl"
        }
    }

    let vert_shader_module = vertex_shader::Shader::load(device.clone())
        .expect("failed to create vertex shader module!");
    let frag_shader_module = fragment_shader::Shader::load(device.clone())
        .expect("failed to create fragment shader module!");

    let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions,
        depth_range: 0.0..1.0,
    };

    let pipeline = GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vert_shader_module.main_entry_point(), ())
        .triangle_list()
        .primitive_restart(false)
        .viewports(vec![viewport])
        .fragment_shader(frag_shader_module.main_entry_point(), ())
        .depth_clamp(false)
        .polygon_mode_fill()
        .line_width(1.0)
        .cull_mode_back()
        .front_face_clockwise()
        .blend_pass_through()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();

    Arc::new(pipeline)
}

fn update_command(
    device: &Arc<Device>,
    graphics_family: QueueFamily,
    vertex_buffer: &Arc<dyn BufferAccess + Send + Sync>,
    framebuffer: &Arc<dyn FramebufferAbstract + Send + Sync>,
    pipeline: &Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
) -> Arc<AutoCommandBuffer> {
    let clear_by = vec![[0.0, 0.0, 0.0, 1.0].into()];
    let dynamic_state = DynamicState::none();
    let mut builder = AutoCommandBufferBuilder::new(device.clone(), graphics_family).unwrap();
    builder
        .begin_render_pass(framebuffer.clone(), SubpassContents::Inline, clear_by)
        .unwrap()
        .draw(pipeline.clone(), &dynamic_state, vec![vertex_buffer.clone()], (), ())
        .unwrap()
        .end_render_pass()
        .unwrap();
    let command = builder.build().unwrap();

    Arc::new(command)
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

impl_vertex!(Vertex, pos, color);

impl Vertex {
    pub fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Vertex { pos, color }
    }
}

fn vertices() -> [Vertex; 3] {
    [
         Vertex::new([0.5, -0.5], [1.0, 0.0, 0.0]),
         Vertex::new([0.5, 0.5], [0.0, 1.0, 0.0]),
         Vertex::new([-0.5, 0.5], [0.0, 0.0, 1.0])
    ]
}

fn create_vertex_buffer(device: &Arc<Device>) -> Arc<dyn BufferAccess + Send + Sync> {
    CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        vertices().iter().cloned()
    ).unwrap()
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(Size::Logical(LogicalSize::new(1024.0, 1024.0)));

    let app = rencan_ui::Application::new(
        VALIDATION_LAYERS,
        window,
        &event_loop,
        create_vertex_buffer,
        create_render_pass,
        create_graphics_pipeline,
        update_command,
    );

    app.run(event_loop)
}
