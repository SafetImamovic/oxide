//#[engine_start] // To be proc_macro
fn engine_start() {}

fn main() -> anyhow::Result<()>
{
        snake::run()?;

        Ok(())
}
