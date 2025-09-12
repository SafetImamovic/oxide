use oxide_macro::oxide_main;
use winit::keyboard::KeyCode;

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let mut engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_tps(1u16)
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.register_behavior(|eng| {
                log::info!("{}", eng.current_tick);
        });

        engine.register_behavior(|eng| {
                log::info!("{}", eng.current_tick);
        });

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
