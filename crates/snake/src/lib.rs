use oxide_macro::oxide_main;

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        oxide::utils::exit::show_exit_message();

        Ok(())
}
