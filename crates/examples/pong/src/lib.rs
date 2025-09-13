use cgmath::{Point3, Vector3};
use oxide_macro::oxide_main;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct Player
{
        pub model_name: &'static str,
        pub position: Point3<f32>,
}

pub struct Ball
{
        pub position: Point3<f32>,
        pub velocity: Vector3<f32>,
}

pub struct PongGame
{
        pub paddle_1: Player,
        pub paddle_2: Player,
        pub ball: Ball,
        pub width: f32,
        pub height: f32,
        pub last_tick: u8,
}

impl PongGame
{
        pub fn new() -> Self
        {
                Self {
                        paddle_1: Player {
                                model_name: "paddle_1",
                                position: Point3::new(-5.0, 0.0, 0.0),
                        },
                        paddle_2: Player {
                                model_name: "paddle_2",
                                position: Point3::new(5.0, 0.0, 0.0),
                        },
                        ball: Ball {
                                position: Point3::new(0.0, 0.0, 0.0),
                                velocity: Vector3::new(4.0, 0.0, 2.0),
                        },
                        width: 10.0,
                        height: 6.0,
                        last_tick: 0,
                }
        }

        pub fn update(
                &mut self,
                delta: f32,
        )
        {
                // Move ball
                self.ball.position += self.ball.velocity * delta;

                // Bounce off top/bottom walls
                if self.ball.position.z >= self.height || self.ball.position.z <= -self.height
                {
                        self.ball.velocity.z = -self.ball.velocity.z;
                }

                // Bounce off paddles
                if (self.ball.position.x <= self.paddle_1.position.x + 0.5
                        && (self.ball.position.z - self.paddle_1.position.z).abs() <= 1.0)
                {
                        self.ball.velocity.x = self.ball.velocity.x.abs();
                }
                if (self.ball.position.x >= self.paddle_2.position.x - 0.5
                        && (self.ball.position.z - self.paddle_2.position.z).abs() <= 1.0)
                {
                        self.ball.velocity.x = -self.ball.velocity.x.abs();
                }

                // Reset ball if out of bounds
                if self.ball.position.x < -self.width || self.ball.position.x > self.width
                {
                        self.ball.position = Point3::new(0.0, 0.0, 0.0);
                        self.ball.velocity = Vector3::new(4.0, 0.0, 2.0);
                }
        }

        pub fn move_paddle(
                &mut self,
                player_id: u8,
                up: bool,
        )
        {
                let paddle = if player_id == 0
                {
                        &mut self.paddle_1
                }
                else
                {
                        &mut self.paddle_2
                };

                let delta = if up { 0.1 } else { -0.1 };
                paddle.position.z += delta;
                // Clamp within bounds
                if paddle.position.z > self.height
                {
                        paddle.position.z = self.height;
                }
                if paddle.position.z < -self.height
                {
                        paddle.position.z = -self.height;
                }
        }
}

#[oxide_main]
pub fn run() -> anyhow::Result<()>
{
        oxide::utils::bootstrap::show_start_message();

        let mut engine = oxide::engine::EngineBuilder::new()
                .with_debug_ui()
                .with_tps(60u16)
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.add_model("paddle_1", "blue_paddle.glb");
        engine.add_model("paddle_2", "blue_paddle.glb");
        engine.add_model("ball", "dodecahedron.glb");

        let mut game = PongGame::new();

        engine.register_behavior(move |eng| {
                let state = match eng.state.as_mut()
                {
                        None => return,
                        Some(s) => s,
                };

                if eng.current_tick == game.last_tick
                {
                        return;
                }

                game.update(1.0 / eng.tps as f32);

                state.models.get_mut("paddle_1").unwrap().position = game.paddle_1.position;
                state.models.get_mut("paddle_2").unwrap().position = game.paddle_2.position;
                state.models.get_mut("ball").unwrap().position = game.ball.position;

                log::info!("last tick: {:?}, current_tick: {:?}", game.last_tick, eng.current_tick);

                if let Some((code, key_state)) = eng.current_key
                {
                        let is_pressed = key_state == ElementState::Pressed;
                        match code
                        {
                                KeyCode::KeyR =>
                                {
                                        if is_pressed
                                        {
                                                game.move_paddle(0, true)
                                        }
                                }
                                KeyCode::KeyF =>
                                {
                                        if is_pressed
                                        {
                                                game.move_paddle(0, false)
                                        }
                                }
                                KeyCode::ArrowUp =>
                                {
                                        if is_pressed
                                        {
                                                game.move_paddle(1, true)
                                        }
                                }
                                KeyCode::ArrowDown =>
                                {
                                        if is_pressed
                                        {
                                                game.move_paddle(1, false)
                                        }
                                }
                                _ =>
                                {}
                        }
                }

                game.last_tick = eng.current_tick;
        });

        let runner = oxide::engine::EngineRunner::new(engine)?;
        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
