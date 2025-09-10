use oxide_macro::oxide_main;
use winit::keyboard::KeyCode;

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let mut engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.add_obj_model("maxwell", "maxwell_the_cat_dingus.glb");

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
