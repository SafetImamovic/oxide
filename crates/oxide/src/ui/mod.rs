use crate::ui::renderer::GuiRenderer;

pub mod renderer;

#[derive(Debug)]
pub struct UiSystem
{
        pub renderer: GuiRenderer,
}
