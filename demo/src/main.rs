use std::sync::Arc;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState, SubpassContents,
};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::vertex::{BufferlessDefinition, BufferlessVertices};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::single_pass_renderpass;
use winit::dpi::{LogicalSize, Size};
use winit::{event_loop::EventLoop, window::WindowBuilder};

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

type ConcreteGP = GraphicsPipeline<
    BufferlessDefinition,
    Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>,
    Arc<dyn RenderPassAbstract + Send + Sync + 'static>,
>;

fn create_graphics_pipeline(
    device: &Arc<Device>,
    swap_chain_extent: [u32; 2],
    render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>,
) -> Arc<ConcreteGP> {
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
        .vertex_input(BufferlessDefinition {})
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(Size::Logical(LogicalSize::new(512.0, 512.0)));

    let mut app = rencan_ui::Application::new(
        VALIDATION_LAYERS,
        window,
        &event_loop,
        create_render_pass,
        create_graphics_pipeline,
    );
    app.add_command_foreach_framebuffer(|device, graphics_family, framebuffer, pipeline| {
        let vertices = BufferlessVertices {
            vertices: 3,
            instances: 1,
        };
        let clear_by = vec![[0.0, 0.0, 0.0, 1.0].into()];
        let dynamic_state = DynamicState::none();
        let mut builder = AutoCommandBufferBuilder::new(device.clone(), graphics_family).unwrap();
        builder
            .begin_render_pass(framebuffer.clone(), SubpassContents::Inline, clear_by)
            .unwrap()
            .draw(pipeline.clone(), &dynamic_state, vertices, (), ())
            .unwrap()
            .end_render_pass()
            .unwrap();
        let command = builder.build().unwrap();

        Arc::new(command)
    });

    app.run(event_loop)
}
