use oxide_macro::oxide_main;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum Direction
{
        Up,
        Down,
        Left,
        Right,
        None,
}

pub struct Snake
{
        pub direction: Direction,
        pub head: &'static str,
}

impl Snake
{
        pub fn new(head: &'static str) -> Self
        {
                Self {
                        direction: Direction::None,
                        head,
                }
        }

        pub fn change_direction(
                &mut self,
                k: &(KeyCode, ElementState),
        )
        {
                if k.1 != ElementState::Pressed
                {
                        return;
                }

                match k.0
                {
                        KeyCode::ArrowUp =>
                        {
                                if self.direction != Direction::Down
                                {
                                        self.direction = Direction::Up;
                                }
                        }
                        KeyCode::ArrowDown =>
                        {
                                if self.direction != Direction::Up
                                {
                                        self.direction = Direction::Down;
                                }
                        }
                        KeyCode::ArrowLeft =>
                        {
                                if self.direction != Direction::Right
                                {
                                        self.direction = Direction::Left;
                                }
                        }
                        KeyCode::ArrowRight =>
                        {
                                if self.direction != Direction::Left
                                {
                                        self.direction = Direction::Right;
                                }
                        }
                        KeyCode::Enter =>
                        {
                                self.direction = Direction::None;
                        }
                        _ =>
                        {}
                }
        }

        pub fn move_snake(
                &mut self,
                head: &mut oxide::model::Model,
        )
        {
                match self.direction
                {
                        Direction::Up =>
                        {
                                head.position.z -= 0.25;
                        }
                        Direction::Down =>
                        {
                                head.position.z += 0.25;
                        }
                        Direction::Left =>
                        {
                                head.position.x -= 0.25;
                        }
                        Direction::Right =>
                        {
                                head.position.x += 0.25;
                        }
                        Direction::None =>
                        {
                                head.position.x = 0.0;
                                head.position.z = 0.0;
                        }
                }
        }
}

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let mut engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_tps(20u16)
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.add_model("snake_head", "dodecahedron.glb");

        let mut snake = Snake::new("snake_head");

        engine.register_behavior(|eng| {
                log::info!("{}", eng.current_tick);
        });

        engine.register_behavior(move |eng| {
                match eng.current_key
                {
                        None =>
                        {}
                        Some(k) => snake.change_direction(&k),
                }

                let mut snake_head = eng
                        .state
                        .as_mut()
                        .unwrap()
                        .models
                        .get_mut("snake_head")
                        .unwrap();

                snake.move_snake(&mut snake_head);
        });

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
