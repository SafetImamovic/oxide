use cgmath::{Deg, Euler, Point3, Quaternion, Rad, Rotation3, Vector3};
use oxide::camera::{Camera, Projection};
use oxide_macro::oxide_main;
use std::collections::HashMap;
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
        pub is_init: bool,
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
                        width: 6.0,
                        height: 6.0,
                        last_tick: 0,
                        is_init: false,
                }
        }

        pub fn init(
                &mut self,
                camera: &mut Camera,
                models: &mut HashMap<String, oxide::model::Model>,
        )
        {
                log::info!("Initializing Pong");

                camera.locked_in = false;

                camera.core.position = Point3::new(0.0, 40.0, -0.6);
                camera.core.pitch = Deg(-89.0).into();
                camera.core.yaw = Deg(90.0).into();
                camera.config.fovy = Deg(17.0);

                let rot_x = Deg(90.0);
                let rot_y_1 = Deg(80.0);
                let rot_y_2 = Deg(100.0);
                let rot_z = Deg(0.0);

                let euler_1 = Euler {
                        x: rot_x,
                        y: rot_y_1,
                        z: rot_z,
                };

                let euler_2 = Euler {
                        x: rot_x,
                        y: rot_y_2,
                        z: rot_z,
                };

                let quat_1: Quaternion<f32> = Quaternion::from(euler_1);
                let quat_2: Quaternion<f32> = Quaternion::from(euler_2);

                models.get_mut("bg").unwrap().position.y = -90.0;

                models.get_mut("bg").unwrap().set_rotation_speed(0, 2.0);
                models.get_mut("bg").unwrap().set_rotation_speed(1, 2.0);
                models.get_mut("bg").unwrap().set_rotation_speed(2, 2.0);
                models.get_mut("bg").unwrap().is_spinning = true;

                models.get_mut("paddle_1").unwrap().rotation = quat_2;

                models.get_mut("paddle_1")
                        .unwrap()
                        .set_rotation_speed(2, 720.0);
                models.get_mut("paddle_1").unwrap().is_spinning = true;

                models.get_mut("paddle_2")
                        .unwrap()
                        .set_rotation_speed(2, 720.0);
                models.get_mut("paddle_2").unwrap().is_spinning = true;

                models.get_mut("ball").unwrap().set_rotation_speed(0, 170.0);
                models.get_mut("ball").unwrap().set_rotation_speed(1, 320.0);
                models.get_mut("ball").unwrap().set_rotation_speed(2, 720.0);

                models.get_mut("ball").unwrap().is_spinning = true;

                models.get_mut("paddle_2").unwrap().rotation = quat_1;

                models.get_mut("ball").unwrap().scale = cgmath::Vector3::new(0.2, 0.2, 0.2);

                self.is_init = true;
        }

        pub fn update(
                &mut self,
                delta: f32,
        )
        {
                self.ball.position += self.ball.velocity * delta;

                if self.ball.position.z >= self.height || self.ball.position.z <= -self.height
                {
                        self.ball.velocity.z = -self.ball.velocity.z;
                }

                // Bounce off paddle 1
                if (self.ball.position.x <= self.paddle_1.position.x + 0.5
                        && (self.ball.position.z - self.paddle_1.position.z).abs() <= 1.0)
                {
                        self.ball.velocity.x = self.ball.velocity.x.abs();

                        // Calculate hit position relative to paddle center
                        let offset = self.ball.position.z - self.paddle_1.position.z;
                        let normalized_offset = offset / 1.0; // since paddle "half-height" ~ 1.0

                        // Add angle effect (scale factor controls steepness)
                        self.ball.velocity.z = normalized_offset * 5.0;

                        // Speed up slightly
                        self.ball.velocity *= 1.05;
                }

                // Bounce off paddle 2
                if (self.ball.position.x >= self.paddle_2.position.x - 0.5
                        && (self.ball.position.z - self.paddle_2.position.z).abs() <= 1.0)
                {
                        self.ball.velocity.x = -self.ball.velocity.x.abs();

                        let offset = self.ball.position.z - self.paddle_2.position.z;
                        let normalized_offset = offset / 1.0;

                        self.ball.velocity.z = normalized_offset * 5.0;

                        self.ball.velocity *= 1.05;
                }

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

                let delta = if up { 0.05 } else { -0.05 };
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
                .with_tps(144u16)
                .with_toggle(KeyCode::Tab)?
                .build()?;

        engine.add_model("bg", "forest_2_by_creepercoastal.glb");
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

                if !game.is_init
                {
                        game.init(&mut state.camera, &mut state.models);
                }

                if eng.current_tick == game.last_tick
                {
                        return;
                }

                game.update(1.0 / eng.tps as f32);

                state.models.get_mut("paddle_1").unwrap().position = game.paddle_1.position;
                state.models.get_mut("paddle_2").unwrap().position = game.paddle_2.position;
                state.models.get_mut("ball").unwrap().position = game.ball.position;

                if eng.pressed_keys.contains(&KeyCode::KeyR)
                {
                        game.move_paddle(1, true);
                }
                if eng.pressed_keys.contains(&KeyCode::KeyF)
                {
                        game.move_paddle(1, false);
                }
                if eng.pressed_keys.contains(&KeyCode::ArrowUp)
                {
                        game.move_paddle(0, true);
                }
                if eng.pressed_keys.contains(&KeyCode::ArrowDown)
                {
                        game.move_paddle(0, false);
                }
                if eng.pressed_keys.contains(&KeyCode::Enter)
                {
                        game.init(&mut state.camera, &mut state.models);
                }

                game.last_tick = eng.current_tick;

                log::info!("Tick: {}", eng.current_tick);
        });

        let runner = oxide::engine::EngineRunner::new(engine)?;
        runner.run()?;

        oxide::utils::exit::show_exit_message();

        Ok(())
}
