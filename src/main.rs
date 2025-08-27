fn main()
{
        let runner = oxide::engine::run().unwrap();
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
