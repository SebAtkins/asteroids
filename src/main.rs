#![allow(non_snake_case)]

use std::f32::consts::PI;

use rand::prelude::*;
use raylib::consts::KeyboardKey::*;
use raylib::ffi::{GetFrameTime, GetWorldToScreen2D};
use raylib::prelude::*;

const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
const PLAYER_HITBOX: f32 = 10.0;
const METEOR_HITBOX: f32 = 20.0;

struct Game {
    player: Player,
    meteors: Vec<Ball>,
    gameOver: bool,
    timer: f32,
    background: Color,
    camera: Camera2D,
    rand: ThreadRng,
    meteorTexture: Image,
    meteorSpawn: f32,
}

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

    fn update(&mut self) {
        self.position += Vector2::new(
            self.speed * (self.rotation * PI / 180.0).cos(),
            self.speed * (self.rotation * PI / 180.0).sin(),
        );
    }
}

fn main() {
    // Definitions
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("Space Game")
        .vsync()
        .build();

    let img = Image::load_image("src/tex/PNG/default/ship_J.png").unwrap();
    let boost = Image::load_image("src/tex/PNG/default/effect_yellow.png").unwrap();
    let asteroid = Image::load_image("src/tex/PNG/default/meteor_detailedLarge.png").unwrap();

    let mut game = Game {
        player: Player {
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
        },
        meteors: Vec::new(),
        gameOver: false,
        timer: 0.0,
        background: Color::from_hex("15203b").unwrap(),
        camera: Camera2D {
            offset: Vector2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
            rotation: 0.0,
            target: Vector2::new(0.0, 0.0),
            zoom: 1.0,
        },
        rand: rand::thread_rng(),
        meteorTexture: asteroid,
        meteorSpawn: 1.0,
    };

    game.meteors.push(Ball {
        position: Vector2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
        speed: 0.0,
        rotation: 45.0,
        tint: Color::GREEN,
        texture: rl
            .load_texture_from_image(&thread, &game.meteorTexture)
            .unwrap(),
    });

    game.meteors.push(Ball {
        position: Vector2::new(150.0, 150.0),
        speed: 0.0,
        rotation: 20.0,
        tint: Color::GREEN,
        texture: rl
            .load_texture_from_image(&thread, &game.meteorTexture)
            .unwrap(),
    });

    // Main loop
    while !rl.window_should_close() {
        // Spawn asteroids
        spawnAsteroids(&mut rl, &mut game, &thread);

        // Handle movement and check for collisions
        mainLoop(&rl, &mut game);

        // Draw player view
        drawGame(&mut rl, &game, &thread);
    }
}

fn spawnAsteroids(rl: &mut RaylibHandle, game: &mut Game, thread: &RaylibThread) {
    // Check if meteor should spawn
    if game.meteorSpawn <= 0.0 {
        // Generate direction of spawn from player
        let direction: i32 = game.rand.gen_range(0..361);

        // Generate spawn position of meteor
        let pos = Vector2::new(
            game.player.position.x + 250.0 * (direction as f32 * PI / 180.0).cos(),
            game.player.position.y + 250.0 * (direction as f32 * PI / 180.0).sin(),
        );

        // Instantiate meteor
        game.meteors.push(Ball {
            position: pos,
            speed: game.rand.gen_range(2..5) as f32,
            rotation: 360.0 - direction as f32 + game.rand.gen_range(-20..20) as f32,
            tint: Color::BROWN,
            texture: rl
                .load_texture_from_image(&thread, &game.meteorTexture)
                .unwrap(),
        });

        // Reset meteor spawn
        game.meteorSpawn = 1.0;
    } else {
        unsafe { game.meteorSpawn = game.meteorSpawn - GetFrameTime() };
    }
}

fn mainLoop(rl: &RaylibHandle, game: &mut Game) {
    // Update player
    game.player.update(rl);

    // Update meteors
    for i in 0..game.meteors.len() {
        game.meteors[i].update();
    }

    // Check for player meteor collisions
    for i in 0..game.meteors.len() {
        if check_collision_circles(
            game.player.position,
            PLAYER_HITBOX,
            game.meteors[i].position,
            METEOR_HITBOX,
        ) {
            game.meteors.remove(i);
            game.gameOver = true;
        }
    }

    // Update timer
    unsafe { game.timer = game.timer + GetFrameTime() }

    // Update camera position
    game.camera.target = game.player.position;
}

fn drawGame(rl: &mut RaylibHandle, game: &Game, thread: &RaylibThread) {
    // Draw background
    let mut d = rl.begin_drawing(&thread);
    d.clear_background(game.background);

    // Draw player
    game.player.draw(&mut d, game.camera);

    // Draw meteors
    for x in &game.meteors {
        x.draw(&mut d, game.camera);
    }

    // Draw timer
    d.draw_text(&(game.timer as i32).to_string(), 10, 10, 40, Color::WHITE);
}
