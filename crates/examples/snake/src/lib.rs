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

        Ok(())
}
