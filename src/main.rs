#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::f32::consts::PI;

use raylib::consts::KeyboardKey::*;
use raylib::ffi::{GetFrameTime, GetWorldToScreen2D};
use raylib::prelude::*;

const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;

struct Player {
    position: Vector2,
    velocity: Vector2,
    speed: f32,
    rotationSpeed: f32,
    rotation: f32,
    tint: Color,
    texture: Texture2D,
    maxSpeed: f32,
    boostTex: Texture2D,
    drawBoost: bool,
}

impl Player {
    fn draw(&self, d: &mut RaylibDrawHandle, cam: Camera2D) {
        let pos = unsafe { GetWorldToScreen2D(self.position.into(), cam.into()) };

        if self.drawBoost {
            d.draw_texture_pro(
                &self.boostTex,
                Rectangle::new(
                    0.0,
                    0.0,
                    self.boostTex.width as f32,
                    self.boostTex.height as f32,
                ),
                Rectangle::new(
                    pos.x - 48.0 * (self.rotation * PI / 180.0).cos(),
                    pos.y - 48.0 * (self.rotation * PI / 180.0).sin(),
                    self.boostTex.width() as f32,
                    self.boostTex.height() as f32,
                ),
                Vector2::new(
                    self.texture.width() as f32 / 2.0,
                    self.texture.height() as f32 / 2.0,
                ),
                self.rotation + 90.0,
                self.tint,
            );
        }
        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(
                0.0,
                0.0,
                self.texture.width as f32,
                self.texture.height as f32,
            ),
            Rectangle::new(
                pos.x,
                pos.y,
                self.texture.width() as f32,
                self.texture.height() as f32,
            ),
            Vector2::new(
                self.texture.width() as f32 / 2.0,
                self.texture.height() as f32 / 2.0,
            ),
            self.rotation + 90.0,
            self.tint,
        );
    }

    fn update(&mut self, rl: &RaylibHandle) {
        // Handle rotation
        if rl.is_key_down(KEY_D) {
            self.rotation += self.rotationSpeed;
        }
        if rl.is_key_down(KEY_A) {
            self.rotation -= self.rotationSpeed;
        }

        // Accelerate
        if rl.is_key_down(KEY_W) {
            self.velocity += Vector2::new(
                self.speed * (self.rotation * PI / 180.0).cos(),
                self.speed * (self.rotation * PI / 180.0).sin(),
            );
            self.drawBoost = true;
        } else {
            self.drawBoost = false;
        }

        // Cap speed
        if self.velocity.length_sqr() > self.maxSpeed {
            let normVel = self.velocity.normalized();

            self.velocity = normVel * self.maxSpeed;
        }

        self.position += self.velocity;
    }
}

struct Ball {
    position: Vector2,
    speed: f32,
    rotation: f32,
    tint: Color,
    texture: Texture2D,
}

impl Ball {
    fn draw(&self, d: &mut RaylibDrawHandle, cam: Camera2D) {
        let pos = unsafe { GetWorldToScreen2D(self.position.into(), cam.into()) };

        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(
                0.0,
                0.0,
                self.texture.width as f32,
                self.texture.height as f32,
            ),
            Rectangle::new(
                pos.x,
                pos.y,
                self.texture.width() as f32,
                self.texture.height() as f32,
            ),
            Vector2::new(
                self.texture.width() as f32 / 2.0,
                self.texture.height() as f32 / 2.0,
            ),
            self.rotation + 90.0,
            self.tint,
        );
    }
}

fn main() {
    // Definitions
    let backgd = Color::from_hex("15203b").unwrap();

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Space Game")
        .vsync()
        .build();

    let img = Image::load_image("src/tex/PNG/default/ship_J.png").unwrap();
    let boost = Image::load_image("src/tex/PNG/default/effect_yellow.png").unwrap();
    let asteroid = Image::load_image("src/tex/PNG/default/meteor_detailedLarge.png").unwrap();

    let mut timer: f32 = 0.0;

    let mut player = Player {
        position: Vector2::new(0.0, 0.0),
        velocity: Vector2::new(0.0, 0.0),
        speed: 0.05,
        rotationSpeed: 2.0,
        rotation: -90.0,
        texture: rl.load_texture_from_image(&thread, &img).unwrap(),
        tint: Color::WHITE,
        maxSpeed: 2.0,
        boostTex: rl.load_texture_from_image(&thread, &boost).unwrap(),
        drawBoost: false,
    };

    let ball1 = Ball {
        position: Vector2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
        speed: 20.0,
        rotation: 45.0,
        tint: Color::WHITE,
        texture: rl.load_texture_from_image(&thread, &asteroid).unwrap(),
    };
    let ball2 = Ball {
        position: Vector2::new(150.0, 150.0),
        speed: 3.0,
        rotation: 20.0,
        tint: Color::GREEN,
        texture: rl.load_texture_from_image(&thread, &asteroid).unwrap(),
    };

    let mut camera = Camera2D {
        offset: Vector2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
        rotation: 0.0,
        target: Vector2::new(player.position.x, player.position.y),
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        // Update player
        player.update(&rl);

        // Update timer
        unsafe { timer = timer + GetFrameTime() }

        camera.target = player.position;

        // Draw
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(backgd);

        player.draw(&mut d, camera);
        ball1.draw(&mut d, camera);
        ball2.draw(&mut d, camera);
        d.draw_text(&(timer as i32).to_string(), 10, 10, 40, Color::WHITE);
    }
}
