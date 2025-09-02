use oxide::geometry::mesh::{Mesh, Primitive};
use oxide_macro::oxide_main;
use winit::keyboard::KeyCode;

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_toggle(KeyCode::Tab)?
                .build()?;

        let _mesh_pentagon = Mesh::basic("pentagon", Primitive::Pentagon);

        let _mesh_square = Mesh::basic("square", Primitive::Square);

        let _mesh_triangle = Mesh::basic("triangle", Primitive::Triangle);

        let hexagon = Mesh::generate_n_gon(128, 0.75);

        {
                let mut resources = engine.resources.lock().unwrap_or_else(|e| e.into_inner());

                resources.add_mesh(hexagon);
        }

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
