use serde::Deserialize;

/// Top level Configuration struct for oxide.
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
        /// Constructs a new `Config` object from the configuration
        /// `config.toml` file.
        ///
        /// TODO: Path is hardcoded at this moment to "../config.toml",
        /// this means that the content from the text file is included within
        /// the library during the compilation step rather than being
        /// decoded at runtime.
        pub fn from_file() -> anyhow::Result<Self>
        {
                let cfg = std::include_str!("../config.toml").to_string();

                log::info!("Config: {:?}", cfg);

                let config: Config = toml::from_str(&cfg)?;

                Ok(config)
        }
}

#[allow(clippy::derivable_impls)]
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
