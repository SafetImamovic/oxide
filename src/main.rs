use oxide::engine::EngineHandler;

pub struct App {}

impl EngineHandler for App
{
        fn setup() -> oxide::engine::EngineRunner
        {
                log::info!("Setting up engine!");

                let engine = oxide::engine::EngineBuilder::new().build().unwrap();

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
