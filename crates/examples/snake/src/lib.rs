use cgmath::Vector3;
use winit::keyboard::KeyCode;
use oxide::engine::{EngineBuilder, EngineRunner};
use oxide::geometry::mesh::{Mesh, Primitive};
use oxide_macro::oxide_main;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        oxide::utils::exit::show_exit_message();

        let mut engine = EngineBuilder::new()
            .with_debug_ui()
            .with_toggle(KeyCode::Tab)?
            .build()?;

        {
                let mut res = engine.resources.lock().unwrap_or_else(|_| panic!("Failed to lock resources"));

                res.add_mesh(Mesh::basic("square" , Primitive::Square));
        }

        const SPEED: f32 = 0.03;

        for (i, key) in [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD].iter().enumerate() {
                let mut direction: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

                let sign = if i % 2 == 0 { 1 } else { -1 };

                if i == 0 || i == 1 {direction.y = sign as f32 * SPEED; direction.x = 0.0}
                if i == 2 || i == 3 {direction.x = -sign as f32 * SPEED; direction.y = 0.0}

                engine.input().on_key(*key, move | _state, resources | {
                         let mesh = resources.meshes.get_mut(0).unwrap();

                         mesh.direction = direction;
                         mesh.needs_upload = true;
                });
        }

        let runner = EngineRunner::new(engine)?;

        runner.run()?;

        Ok(())
}
