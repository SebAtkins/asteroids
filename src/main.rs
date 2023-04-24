#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::f32::consts::PI;

use raylib::consts::KeyboardKey::*;
use raylib::ffi::GetWorldToScreen2D;
use raylib::prelude::*;

use rand::prelude::*;

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
    particleCooldown: i32,
    rng: ThreadRng,
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
                    pos.x,
                    pos.y,
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
        } else {
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

    fn update(&mut self, rl: &RaylibHandle, particles: &mut ParticleHandler) {
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

            // Spawn particle
            /*
            self.particleCooldown -= 1;
            if self.particleCooldown == 0 {
                // Create background particle
                particles.push(Particle {
                    position: self.position,
                    rotation: self.rotation + (self.rng.gen::<f32>() * 1000.0) % 360.0,
                    scale: 20.0,
                    color: Color::ORANGE,
                    lifetime: 200,
                });

                // Create foreground particle
                particles.push(Particle {
                    position: self.position,
                    rotation: self.rotation - self.rng.gen_range(0.0..361.0),
                    scale: 11.5,
                    color: Color::BROWN,
                    lifetime: 200,
                });

                particles.push(Particle {
                    position: self.position,
                    rotation: self.rotation + (self.rng.gen::<f32>() * 1000.0) % 360.0,
                    scale: 2.5,
                    color: Color::BLACK,
                    lifetime: 200,
                });

                self.particleCooldown = 20;
            }
            */

            // Draw booster
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

struct Particle {
    position: Vector2,
    rotation: f32,
    scale: f32,
    color: Color,
    lifetime: i32,
}

impl Particle {
    fn draw(&mut self, d: &mut RaylibDrawHandle, cam: Camera2D) {
        let pos = unsafe { GetWorldToScreen2D(self.position.into(), cam.into()) };

        d.draw_rectangle_pro(
            Rectangle::new(pos.x, pos.y, self.scale, self.scale),
            Vector2::new(self.scale / 2.0, self.scale / 2.0),
            self.rotation,
            self.color,
        );

        self.lifetime -= 1;
    }

    fn die(&self) -> bool {
        if self.lifetime == 0 {
            true
        } else {
            false
        }
    }
}

struct ParticleHandler {
    particles: VecDeque<Particle>,
    maxParticles: usize,
}

impl ParticleHandler {
    fn push(&mut self, particle: Particle) {
        if self.particles.len() == self.maxParticles {
            self.pop();
        }
        self.particles.push_back(particle);
    }

    fn pop(&mut self) {
        self.particles.pop_front();
    }

    fn drawParticles(&mut self, d: &mut RaylibDrawHandle, cam: Camera2D) {
        let mut toPop: i32 = 0;

        for particle in &mut self.particles {
            particle.draw(d, cam);
            if particle.die() {
                toPop += 1;
            }
        }

        for _ in 0..toPop {
            self.pop();
        }
    }
}

struct Ball {
    position: Vector2,
    speed: f32,
    radius: f32,
    color: Color,
}

impl Ball {
    fn draw(&self, d: &mut RaylibDrawHandle, cam: Camera2D) {
        unsafe {
            d.draw_circle_v(
                GetWorldToScreen2D(self.position.into(), cam.into()),
                self.radius * cam.zoom,
                self.color,
            )
        }
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

    let img = Image::load_image("src/tex/PNG/default/ship_E.png").unwrap();
    let boost = Image::load_image("src/tex/PNG/default/boostMoment.png").unwrap();

    let mut player = Player {
        position: Vector2::new(0.0, 0.0),
        velocity: Vector2::new(0.0, 0.0),
        speed: 0.05,
        rotationSpeed: 2.0,
        rotation: -90.0,
        texture: rl.load_texture_from_image(&thread, &img).unwrap(),
        tint: Color::WHITE,
        maxSpeed: 2.0,
        particleCooldown: 20,
        rng: rand::thread_rng(),
        boostTex: rl.load_texture_from_image(&thread, &boost).unwrap(),
        drawBoost: false,
    };

    let ball1 = Ball {
        position: Vector2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
        speed: 3.0,
        radius: 20.0,
        color: Color::GREEN,
    };
    let ball2 = Ball {
        position: Vector2::new(150.0, 150.0),
        speed: 3.0,
        radius: 20.0,
        color: Color::GREEN,
    };

    let mut camera = Camera2D {
        offset: Vector2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
        rotation: 0.0,
        target: Vector2::new(player.position.x, player.position.y),
        zoom: 1.0,
    };

    let mut particles = ParticleHandler {
        particles: VecDeque::new(),
        maxParticles: 100,
    };

    while !rl.window_should_close() {
        // Update player
        player.update(&rl, &mut particles);

        camera.target = player.position;

        // Draw
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(backgd);

        particles.drawParticles(&mut d, camera);

        player.draw(&mut d, camera);
        ball1.draw(&mut d, camera);
        ball2.draw(&mut d, camera);
        d.draw_fps(10, 10);
    }
}
