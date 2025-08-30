#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;

pub fn show_start_message()
{
        let oxide_string = r#"

      ░██████   ░██    ░██ ░██████░███████   ░██████████ 
     ░██   ░██   ░██  ░██    ░██  ░██   ░██  ░██         
    ░██     ░██   ░██░██     ░██  ░██    ░██ ░██         
    ░██     ░██    ░███      ░██  ░██    ░██ ░█████████  
    ░██     ░██   ░██░██     ░██  ░██    ░██ ░██         
     ░██   ░██   ░██  ░██    ░██  ░██   ░██  ░██         
      ░██████   ░██    ░██ ░██████░███████   ░██████████ 
                                                            
 Web 3D Render Engine built with wgpu and Rust. 
(ASCII art generated @ https://www.patorjk.com/software/taag/
[font: Terrace])

            "#;

        log::info!("{oxide_string}")
}

pub fn config_logging()
{
        #[cfg(not(target_arch = "wasm32"))]
        {
                env_logger::init();

                log::info!("Running on native.");
        }

        #[cfg(target_arch = "wasm32")]
        {
                console_log::init_with_level(log::Level::Info).unwrap_throw();

                log::info!("Running on wasm32.");
        }
}
