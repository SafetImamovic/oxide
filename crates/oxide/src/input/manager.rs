use std::collections::HashMap;
use derivative::Derivative;
use winit::event::ElementState;
use winit::keyboard::KeyCode;
use crate::resource::Resources;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct InputManager
{
        #[derivative(Debug = "ignore")]
        pub key_actions: HashMap<KeyCode, Vec<Box<dyn Fn(ElementState, &mut Resources)>>>,
}

impl InputManager {
        pub fn new() -> Self {
                Self {
                        key_actions: HashMap::new(),
                }
        }

        pub fn on_key<F>(&mut self, key: KeyCode, callback: F)
        where
            F: 'static + Fn(ElementState, &mut Resources),
        {
                self.key_actions
                    .entry(key)
                    .or_default()
                    .push(Box::new(callback));
        }

        pub fn handle_event(&mut self, key: KeyCode, state: ElementState, resources: &mut Resources) {
                if let Some(actions) = self.key_actions.get(&key) {
                        for action in actions {
                                action(state, resources);
                        }
                }
        }
}