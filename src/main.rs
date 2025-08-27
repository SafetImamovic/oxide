use oxide::{
        engine::EngineHandler,
};
use oxide::geometry::mesh::Mesh;
use oxide::geometry::vertex::Vertex;

pub const PENTAGON: &[Vertex] = &[
        Vertex {
                position: [-0.0868241, 0.49240386, 0.0],
                tex_coords: [0.4131759, 0.00759614],
        }, // A
        Vertex {
                position: [-0.49513406, 0.06958647, 0.0],
                tex_coords: [0.0048659444, 0.43041354],
        }, // B
        Vertex {
                position: [-0.21918549, -0.44939706, 0.0],
                tex_coords: [0.28081453, 0.949397],
        }, // C
        Vertex {
                position: [0.35966998, -0.3473291, 0.0],
                tex_coords: [0.85967, 0.84732914],
        }, // D
        Vertex {
                position: [0.44147372, 0.2347359, 0.0],
                tex_coords: [0.9414737, 0.2652641],
        }, // E
];

pub const P_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub const MESH: Mesh = Mesh {
        vertices: PENTAGON,
        indices: P_INDICES,
};

pub struct App {}

impl EngineHandler for App
{
        fn setup() -> oxide::engine::EngineRunner
        {
                log::info!("Setting up engine!");

                let mut engine = oxide::engine::EngineBuilder::new().build().unwrap();

                engine.add_mesh("Pentagon", MESH);

                oxide::engine::EngineRunner::new(engine).unwrap()
        }
}

fn main()
{
        oxide::engine::run::<App>();
}

/*
let mut engine = Engine::new();


// Load resources
let tex = engine.load_texture("brick.png");
let mesh = engine.load_mesh("cube.obj");

// Create material
let mat = engine.create_material("pbr_shader")
    .with_texture("albedo", tex);

// Scene
engine.draw(mesh, mat, Transform::from_position([0.0, 0.0, -5.0]));

// Effects
engine.add_effect(Effect::Bloom { intensity: 1.2 });
engine.add_effect(Effect::DepthOfField { focus: 0.5 });
engine.add_effect(Effect::ShadowMapping);

// Main loop
engine.run();
*/

//oxide::run().unwrap();
