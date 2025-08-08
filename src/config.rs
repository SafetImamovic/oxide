use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config
{
        #[cfg(target_arch = "wasm32")]
        pub default_canvas_width: u32,

        #[cfg(target_arch = "wasm32")]
        pub default_canvas_height: u32,
}

impl Config
{
        pub fn from_file() -> anyhow::Result<Self>
        {
                let cfg = std::include_str!("../config.toml").to_string();

                log::info!("Config: {:?}", cfg);

                let config: Config = toml::from_str(&cfg)?;

                Ok(config)
        }
}

impl Default for Config
{
        fn default() -> Self
        {
                Self {
                        #[cfg(target_arch = "wasm32")]
                        default_canvas_width: 1280,
                        #[cfg(target_arch = "wasm32")]
                        default_canvas_height: 720,
                }
        }
}
