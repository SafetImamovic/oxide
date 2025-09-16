use oxide_macro::oxide_main;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct Segment
{
        pub prev_pos: cgmath::Vector3<f32>,
        pub pos: cgmath::Vector3<f32>,
}

impl Segment
{
        pub fn store_prev(&mut self)
        {
                self.prev_pos = self.pos;
        }

        pub fn interpolate(
                &self,
                alpha: f32,
        ) -> cgmath::Vector3<f32>
        {
                self.prev_pos * (1.0 - alpha) + self.pos * alpha
        }
}
pub struct SnakeGame
{
        pub grid: Grid,
        pub snake: Snake,
        pub last_tick: u8,
        pub started: bool,
        pub game_over: bool,
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
                        started: false,
                        last_tick: 0,
                        game_over: false,
                }
        }

        pub fn start(
                &mut self,
                model: &mut oxide::model::Model,
        )
        {
                log::info!("Starting");

                let x = self.grid.width / 2;
                let z = self.grid.height / 2;

                self.snake.grid_pos = (x, z);

                self.snake.update_segment_pos();
                self.snake.segment.store_prev();

                model.position = cgmath::Point3::new(x as f32, 0.0, z as f32);
        }

        pub fn is_colliding(&self) -> bool
        {
                if self.snake.grid_pos.0 > self.grid.width
                {
                        return true;
                }
                if self.snake.grid_pos.1 > self.grid.height
                {
                        return true;
                }
                if self.snake.grid_pos.0 == 0
                {
                        return true;
                }
                if self.snake.grid_pos.1 == 0
                {
                        return true;
                }

                false
        }

        pub fn update_grid_pos(&mut self)
        {
                match self.snake.direction
                {
                        Direction::Up =>
                        {
                                self.snake.grid_pos.0 += 1;
                        }
                        Direction::Down =>
                        {
                                self.snake.grid_pos.0 -= 1;
                        }
                        Direction::Left =>
                        {
                                self.snake.grid_pos.1 -= 1;
                        }
                        Direction::Right =>
                        {
                                self.snake.grid_pos.1 += 1;
                        }
                        Direction::None =>
                        {
                                self.snake.grid_pos.0 = self.grid.width / 2;
                                self.snake.grid_pos.1 = self.grid.height / 2;
                        }
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

        pub segment: Segment,
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
                        segment: Segment {
                                prev_pos: cgmath::Vector3::new(0.0, 0.0, 0.0),
                                pos: cgmath::Vector3::new(0.0, 0.0, 0.0),
                        },
                }
        }

        pub fn change_direction(
                &mut self,
                k: &(KeyCode, ElementState),
        )
        {
                /*
                if k.1 != ElementState::Pressed
                {
                        return;
                }
                */

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

        pub fn update_segment_pos(&mut self)
        {
                // log::info!("{:?}", self.grid_pos);

                self.segment.pos.x = self.grid_pos.0 as f32;
                self.segment.pos.z = self.grid_pos.1 as f32;
        }
}

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let mut engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_tps(2u16)
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.add_model("snake_head", "dodecahedron.glb");
        engine.add_model("log_1", "log_photogrammetrised.glb");
        engine.add_model("log_2", "log_photogrammetrised.glb");
        engine.add_model("log_3", "log_photogrammetrised.glb");
        engine.add_model("log_4", "log_photogrammetrised.glb");

        let snake = Snake::new("snake_head", 4f32);

        let mut game = SnakeGame::new(Grid::new(20, 20), snake);

        engine.register_behavior(move |eng| {
                let state = match eng.state.as_mut()
                {
                        None => return,
                        Some(s) => s,
                };

                let snake_head = state.models.get_mut("snake_head").unwrap();

                if !game.started
                {
                        game.start(snake_head);
                        game.started = true;
                }

                let v = game.snake.segment.interpolate(eng.lerp_alpha);
                snake_head.position = cgmath::Point3::new(v.x, v.y, v.z);

                if eng.current_tick != game.last_tick
                {
                        game.snake.segment.store_prev();
                        game.update_grid_pos();
                        game.snake.update_segment_pos();

                        game.last_tick = eng.current_tick;

                        log::info!(
                                "Tick {}, Prev: {:?}, Pos: {:?}",
                                eng.current_tick,
                                game.snake.segment.prev_pos,
                                game.snake.segment.pos
                        );

                        if game.is_colliding()
                        {
                                log::info!("Game Over");

                                snake_head.position = cgmath::Point3::new(0.0, 0.0, 0.0);

                                game.game_over = true;

                                game.snake
                                        .change_direction(&(KeyCode::Enter, ElementState::Pressed));

                                game.snake.segment.store_prev();
                                game.update_grid_pos();
                                game.snake.update_segment_pos();
                        }
                }
        });

        let runner = oxide::engine::EngineRunner::new(engine)?;

        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
