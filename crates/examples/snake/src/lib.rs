use oxide_macro::oxide_main;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct SnakeGame
{
        pub grid: Grid,
        pub snake: Snake,
}

impl SnakeGame
{
        pub fn new(
                grid: Grid,
                snake: Snake,
        ) -> Self
        {
                Self {
                        grid,
                        snake,
                }
        }
}

pub struct Grid
{
        pub width: u8,
        pub height: u8,
}

impl Grid
{
        pub fn new(
                width: u8,
                height: u8,
        ) -> Self
        {
                Self {
                        width,
                        height,
                }
        }
}

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
        pub grid_pos: (u8, u8),
        pub step_speed: f32,
}

impl Snake
{
        pub fn new(
                head: &'static str,
                step_speed: f32,
        ) -> Self
        {
                Self {
                        direction: Direction::None,
                        head,
                        grid_pos: (0, 0),
                        step_speed,
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
                log::info!("{:?}", self.grid_pos);

                match self.direction
                {
                        Direction::Up =>
                        {
                                head.position.z -= self.step_speed;

                                self.grid_pos.0 += 1;
                        }
                        Direction::Down =>
                        {
                                head.position.z += self.step_speed;

                                self.grid_pos.0 -= 1;
                        }
                        Direction::Left =>
                        {
                                head.position.x -= self.step_speed;

                                self.grid_pos.1 -= 1;
                        }
                        Direction::Right =>
                        {
                                head.position.x += self.step_speed;

                                self.grid_pos.1 += 1;
                        }
                        Direction::None =>
                        {
                                head.position.x = 0.0;
                                head.position.z = 0.0;

                                self.grid_pos.0 = 0;
                                self.grid_pos.1 = 0;
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

        let snake = Snake::new("snake_head", 0.5f32);

        let mut game = SnakeGame::new(Grid::new(20, 20), snake);

        engine.register_behavior(|eng| {
                log::info!("{}", eng.current_tick);
        });

        engine.register_behavior(move |eng| {
                match eng.current_key
                {
                        None =>
                        {}
                        Some(k) => game.snake.change_direction(&k),
                }

                let mut snake_head = eng
                        .state
                        .as_mut()
                        .unwrap()
                        .models
                        .get_mut("snake_head")
                        .unwrap();

                game.snake.move_snake(&mut snake_head);
        });

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
