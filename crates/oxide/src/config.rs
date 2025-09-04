use crate::engine::FillMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config
{
        /// Polygon fill mode, depends on the platforms wgpu features.
        pub fill_mode: FillMode,
        pub enable_debug: bool,
        pub debug_toggle_key: Option<u32>,
}

impl Config
{
        pub fn new() -> Self
        {
                Self {
                        fill_mode: FillMode::Fill,
                        enable_debug: false,
                        debug_toggle_key: None,
                }
        }
}
