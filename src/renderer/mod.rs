pub mod graph;
pub mod pipeline;
pub mod renderer;
pub mod resource;
pub mod shader;

pub struct RenderContext
{
        pub renderer: renderer::Renderer,
        pub graph: graph::RenderGraph,
        pub pipelines: pipeline::PipelineManager,
        pub shaders: shader::ShaderManager,
        pub resources: resource::ResourceManager,
}
