use oxide_macro::oxide_main;
use winit::keyboard::KeyCode;

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let resources = oxide::resources::load_resources();
        log::info!("Loading Resources from: {}", resources.display());

        let mut engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.add_obj_model("auto", "free_1975_porsche_911_930_turbo.glb");

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
