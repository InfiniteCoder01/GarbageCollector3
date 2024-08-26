#![feature(proc_macro_hygiene)]
#![feature(custom_inner_attributes)]

use std::fmt::Debug;

use assets::Assets;
use controls::Controls;
use player::Player;
use rand::Rng;
use speedy2d::color::Color;
use speedy2d::dimen::*;
use speedy2d::font::TextLayout;
use speedy2d::window::{VirtualKeyCode, WindowHandler, WindowHelper};
use speedy2d::Graphics2D;
use watch::{interpreter, Watch};
use world::traits::*;

pub mod assets;
pub mod controls;
pub mod player;
pub mod watch;
pub mod world;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_logger::init(wasm_logger::Config::default());
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    let handler = GarbageCollector3::new();

    #[cfg(target_arch = "wasm32")]
    speedy2d::WebCanvas::new_for_id("canvas", handler).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let window = speedy2d::Window::new_centered("GarbageCollector3", (854, 480)).unwrap();
        window.run_loop(handler)
    }
}

struct GarbageCollector3 {
    stopwatch: speedy2d::time::Stopwatch,
    assets: Option<Assets>,
    camera: Vec2,
    controls: Controls,

    level_index: usize,
    introduced: bool,
    dialogue: &'static [&'static str],

    world: world::World,
    player: Player,
    watch: Watch,

    particles: Vec<Particle>,
    weather_particle_timer: f32,

    timer: Option<f32>,
    finished: bool,
}

impl GarbageCollector3 {
    fn new() -> Self {
        let world = world::World::load();
        let player = Player::new(get_player_start_position(&world.level_0.entities));
        Self {
            stopwatch: speedy2d::time::Stopwatch::new().unwrap(),
            assets: None,
            camera: Vec2::ZERO,
            controls: Controls::default(),

            level_index: 0,
            introduced: false,
            dialogue: &[],

            world,
            player,
            watch: Watch::default(),
            particles: Vec::new(),
            weather_particle_timer: 0.0,

            timer: None,
            finished: false,
        }
    }
}

impl WindowHandler for GarbageCollector3 {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        let assets: &Assets = self.assets.get_or_insert_with(|| {
            let assets = Assets::load(graphics);
            self.player.size = UVec2::new(
                assets.player.image.size().x / self.player.frame_count,
                assets.player.image.size().y,
            );
            assets
        });
        let delta_time = self.stopwatch.secs_elapsed() as f32;
        self.stopwatch = speedy2d::time::Stopwatch::new().unwrap();

        if let Some(timer) = &mut self.timer {
            if !self.finished {
                *timer += delta_time;
            }
        }

        let level = &self.world[self.level_index];

        let scale = helper.get_size_pixels().y as f32 / 256.0;
        let screen_size = helper.get_size_pixels().into_f32() / scale;
        if !self.watch.open && self.dialogue.is_empty() {
            self.player.update(delta_time, level, &self.controls);
            for entity in level.entities.entities() {
                if self.player.overlaps(entity) {
                    match entity.entity {
                        world::Entity::EndOfTheLevel(_) => {
                            self.level_index += 1;
                            self.player.position =
                                get_player_start_position(&self.world[self.level_index].entities);
                            self.camera = self.player.position + self.player.size.into_f32() / 2.0
                                - screen_size / 2.0;
                        }

                        world::Entity::Void(_) => {
                            if self.level_index == 0 {
                                if !self.introduced {
                                    self.dialogue = &[
                                    "Hey!",
                                    "I'm Void, and you probably have heard of me.",
                                    "So, I just finished designing this watch...",
                                    "It's not your usual fitness bracelet. It's something more!",
                                    "And I want you to test it...",
                                    "Can you just *run* through this obstacle course I made for you as fast as possible?",
                                    "You might need to *write* some code to unleash bracelet's full potential...",
                                    "Your time starts... Now!",
                                ];
                                    self.introduced = true;
                                }
                            } else {
                                self.finished = true;
                                self.dialogue = &["You did it! It only took you $TIME"];
                            }
                        }
                        _ => (),
                    }
                }
            }
        }

        for particle in &mut self.particles {
            particle.update(level, delta_time);
        }
        self.particles.retain(|particle| particle.lifetime > 0.0);

        self.camera += ((self.player.position + self.player.size.into_f32() / 2.0
            - screen_size / 2.0)
            - self.camera)
            * (1.0 - 0.05_f32.powf(delta_time));
        self.camera.x = self
            .camera
            .x
            .clamp(0.0, level.pixel_size.x as f32 - screen_size.x);
        self.camera.y = self
            .camera
            .y
            .clamp(0.0, level.pixel_size.y as f32 - screen_size.y);
        let mut camera = Camera {
            graphics,
            scale,
            position: self.camera,
        };

        {
            let weather = *watch::interpreter::pywatch::WEATHER.lock().unwrap();
            camera.graphics.clear_screen(match weather {
                interpreter::pywatch::Weather::Sunny => level.bg_color,
                interpreter::pywatch::Weather::Rainy => Color::from_hex_rgb(0x9F9F9F),
                interpreter::pywatch::Weather::Snowy => Color::from_hex_rgb(0xDADADA),
            });

            let (pps, particle_color) = match weather {
                interpreter::pywatch::Weather::Rainy => (
                    0.1 * level.pixel_size.x as f32,
                    Color::from_hex_rgb(0x00057F),
                ),
                interpreter::pywatch::Weather::Snowy => (
                    0.1 * level.pixel_size.x as f32,
                    Color::from_hex_rgb(0xFFFFFF),
                ),
                _ => (0.0, Color::MAGENTA),
            };
            self.weather_particle_timer += delta_time;
            while self.weather_particle_timer > 1.0 / pps {
                self.weather_particle_timer -= 1.0 / pps;
                self.particles.push(Particle::new(
                    Vec2::new(
                        rand::thread_rng().gen_range(0.0..level.pixel_size.x as f32),
                        rand::thread_rng().gen_range(-20.0..20.0_f32),
                    ),
                    Vec2::ZERO,
                    ParticleVisual::Color(particle_color, Vec2::new(1.0, 2.5), false),
                    true,
                ));
            }
            if pps == 0.0 {
                self.weather_particle_timer = 0.0;
            }
        }

        camera.draw_tiles(screen_size, assets, &level.background);
        camera.draw_autotile(screen_size, assets, &level.solid);
        camera.draw_tiles(screen_size, assets, &level.ambient_decorations);
        let level = &mut self.world[self.level_index];
        for entity in level.entities.entities_mut() {
            match &mut entity.entity {
                world::Entity::EndOfTheLevel(eol) => {
                    let pps = 100.0;
                    eol.particle_timer += delta_time;
                    while eol.particle_timer > 1.0 / pps {
                        eol.particle_timer -= 1.0 / pps;
                        let direction =
                            rand::thread_rng().gen_range(0.0..std::f32::consts::PI * 2.0);
                        let velocity = direction.sin_cos();
                        let velocity = Vec2::new(velocity.0, velocity.1)
                            * rand::thread_rng().gen_range(0.0..100.0);
                        self.particles.push(Particle::new(
                            entity.position,
                            velocity,
                            ParticleVisual::Color(Color::WHITE, Vec2::new(2.0, 2.0), true),
                            false,
                        ));
                    }
                }
                world::Entity::Void(void) => {
                    let pps = 100.0;
                    void.particle_timer += delta_time;
                    while void.particle_timer > 1.0 / pps {
                        void.particle_timer -= 1.0 / pps;
                        let direction =
                            rand::thread_rng().gen_range(0.0..std::f32::consts::PI * 2.0);
                        let velocity = direction.sin_cos();
                        let velocity = Vec2::new(velocity.0, velocity.1)
                            * rand::thread_rng().gen_range(0.0..100.0);
                        self.particles.push(Particle::new(
                            entity.position,
                            velocity,
                            ParticleVisual::Texture(rand::thread_rng().gen_range(0..=1)),
                            true,
                        ));
                    }
                }
                world::Entity::Platform(platform) => {
                    let point_true = platform.point_true.into_f32() + Vec2::new(0.5, 0.5);
                    let point_true = (point_true * world::Entities::GRID_SIZE as f32
                        - camera.position)
                        * camera.scale;
                    let point_false = platform.point_false.into_f32() + Vec2::new(0.5, 0.5);
                    let point_false = (point_false * world::Entities::GRID_SIZE as f32
                        - camera.position)
                        * camera.scale;
                    let condition = platform.condition.clone();
                    camera.graphics.draw_line(
                        point_true,
                        point_false,
                        camera.scale,
                        Color::from_hex_rgb(0x52333f),
                    );
                    camera
                        .graphics
                        .draw_circle(point_true, 2.0 * camera.scale, Color::GREEN);
                    camera
                        .graphics
                        .draw_circle(point_false, 2.0 * camera.scale, Color::RED);
                    camera.draw_tile(
                        entity.top_left(),
                        false,
                        UVec2::new(5, 0),
                        UVec2::new(32, 16),
                        &assets.tileset,
                        false,
                        false,
                    );
                    let text = assets.font.layout_text(
                        &condition,
                        10.0 * camera.scale,
                        speedy2d::font::TextOptions::new(),
                    );
                    let size = text.size();
                    camera.graphics.draw_text(
                        point_true - Vec2::new(size.x * 0.5, size.y),
                        Color::BLACK,
                        &text,
                    );
                }
                _ => (),
            }
        }
        self.player.draw(&mut camera, assets, self.introduced);
        camera.draw_tiles(screen_size, assets, &level.foreground);
        for particle in &self.particles {
            particle.draw(&mut camera, assets);
        }
        if self.introduced {
            self.watch.draw(
                helper,
                delta_time,
                &self.controls,
                &mut camera,
                assets,
                level,
                &self.player,
            );
        }
        if let Some(line) = self.dialogue.first() {
            let mut position = helper.get_size_pixels().into_f32();
            position.x *= 0.5;
            position.y *= 0.5;
            let text = assets.font.layout_text(
                &line.replace(
                    "$TIME",
                    &self.timer.map_or("???".to_owned(), |time| {
                        format!(
                            "{:02}:{:02}:{:01.2}",
                            (time / 60.0 / 60.0) as i32,
                            (time / 60.0) as i32 % 60,
                            time % 60.0
                        )
                    }),
                ),
                12.0 * camera.scale,
                speedy2d::font::TextOptions::new()
                    .with_wrap_to_width(80.0 * camera.scale, speedy2d::font::TextAlignment::Left),
            );
            let size = text.size();
            let border = 5.0;
            let mut rect_size = size;
            rect_size.x = rect_size.x.max(80.0 * camera.scale);
            rect_size.y = rect_size.y.max(60.0 * camera.scale);
            // * Outer border
            let mut rect = speedy2d::shape::RoundRect::new(
                position - rect_size / 2.0 - Vec2::new(1.0, 1.0) * 10.0 * camera.scale,
                position + rect_size / 2.0 + Vec2::new(1.0, 1.0) * 10.0 * camera.scale,
                10.0 * camera.scale,
            );
            camera
                .graphics
                .draw_rounded_rectangle(&rect, Color::from_hex_rgb(0xdfe0e8));
            // * Inner border
            rect = speedy2d::shape::RoundRect::new(
                rect.top_left() + Vec2::new(1.0, 1.0) * border / 2.0 * camera.scale,
                rect.bottom_right() - Vec2::new(1.0, 1.0) * border / 2.0 * camera.scale,
                rect.radius() - border / 2.0 * camera.scale,
            );
            camera
                .graphics
                .draw_rounded_rectangle(&rect, Color::from_hex_rgb(0x686f99));
            // * Inside
            rect = speedy2d::shape::RoundRect::new(
                rect.top_left() + Vec2::new(1.0, 1.0) * border / 2.0 * camera.scale,
                rect.bottom_right() - Vec2::new(1.0, 1.0) * border / 2.0 * camera.scale,
                rect.radius() - border / 2.0 * camera.scale,
            );
            camera
                .graphics
                .draw_rounded_rectangle(&rect, Color::from_hex_rgb(0x3d2936));

            use speedy2d::numeric::RoundFloat;
            camera
                .graphics
                .draw_text((position - size / 2.0).round(), Color::WHITE, &text);
            if self.controls.dialogue_next() {
                self.dialogue = &self.dialogue[1..];
                if self.dialogue.is_empty() {
                    if self.level_index == 0 {
                        self.timer = Some(0.0);
                    }
                    level
                        .entities
                        .entities_mut()
                        .retain(|entity| !matches!(entity.entity, world::Entity::Void(_)));
                }
            }
        }
        self.controls.reset();
        helper.request_redraw();
    }

    fn on_key_down(
        &mut self,
        _helper: &mut WindowHelper<()>,
        virtual_key_code: Option<speedy2d::window::VirtualKeyCode>,
        _scancode: speedy2d::window::KeyScancode,
    ) {
        if let Some(virtual_key_code) = virtual_key_code {
            self.controls.pressed.insert(virtual_key_code, true);
            self.controls.jpressed.insert(virtual_key_code, true);
        }
    }

    fn on_key_up(
        &mut self,
        _helper: &mut WindowHelper<()>,
        virtual_key_code: Option<speedy2d::window::VirtualKeyCode>,
        _scancode: speedy2d::window::KeyScancode,
    ) {
        if let Some(virtual_key_code) = virtual_key_code {
            self.controls.pressed.insert(virtual_key_code, false);
        }
    }

    fn on_keyboard_modifiers_changed(
        &mut self,
        _helper: &mut WindowHelper<()>,
        state: speedy2d::window::ModifiersState,
    ) {
        self.controls.mods = state.clone();
    }

    fn on_mouse_move(&mut self, _helper: &mut WindowHelper<()>, position: Vec2) {
        self.controls.mouse_pos = position;
    }

    fn on_mouse_button_down(
        &mut self,
        _helper: &mut WindowHelper<()>,
        button: speedy2d::window::MouseButton,
    ) {
        self.controls.mouse_buttons.insert(button, true);
    }

    fn on_keyboard_char(&mut self, _helper: &mut WindowHelper<()>, unicode_codepoint: char) {
        if (unicode_codepoint as u32) < 9 {
            return;
        }
        self.controls.typed_text.push(unicode_codepoint);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleVisual {
    Color(Color, Vec2, bool),
    Texture(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    position: Vec2,
    velocity: Vec2,
    visual: ParticleVisual,
    gravity: bool,
    lifetime: f32,
}

impl Particle {
    pub fn new(position: Vec2, velocity: Vec2, visual: ParticleVisual, gravity: bool) -> Self {
        Self {
            position,
            velocity,
            visual,
            gravity,
            lifetime: 5.0,
        }
    }

    pub fn update(&mut self, level: &world::Level, delta_time: f32) {
        self.position += self.velocity * delta_time;
        if self.gravity {
            self.velocity.y += 18.0 * 32.0 * delta_time;
        }
        self.velocity *= 0.9;
        let tile_pos = IVec2::new(
            (self.position.x / level.solid.grid_size().x as f32).floor() as i32,
            (self.position.y / level.solid.grid_size().y as f32).floor() as i32,
        );
        if let Some(world::SolidTile::Ground) = level.solid.get(tile_pos) {
            self.lifetime = -1.0
        }
        if let Some(tile) = level.background.get(tile_pos) {
            if tile.position.x == 7 {
                self.lifetime = -1.0;
            }
        }
        self.lifetime -= delta_time;
    }

    pub fn draw(&self, camera: &mut Camera, assets: &Assets) {
        match self.visual {
            ParticleVisual::Color(color, size, fading) => {
                let position = (self.position - camera.position) * camera.scale;
                let size = size * camera.scale;
                let alpha = if fading { self.lifetime / 5.0 } else { 1.0 };
                camera.graphics.draw_rectangle(
                    speedy2d::shape::Rect::new(position - size / 2.0, position + size / 2.0),
                    Color::from_rgba(color.r(), color.g(), color.b(), color.a() * alpha),
                );
            }
            ParticleVisual::Texture(texture) => {
                let size = Vec2::new(3.0, 4.0);
                camera.draw_tile(
                    self.position - size / 2.0,
                    false,
                    UVec2::new_x(texture as _),
                    size.into_u32(),
                    &assets.particles,
                    false,
                    false,
                )
            }
        }
    }
}

pub struct Camera<'a> {
    pub graphics: &'a mut Graphics2D,
    pub scale: f32,
    pub position: Vec2,
}

impl Camera<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn draw_tile(
        &mut self,
        mut pos: Vec2,
        gui: bool,
        tile: UVec2,
        tile_size: UVec2,
        image: &speedy2d::image::ImageHandle,
        flip_h: bool,
        flip_v: bool,
    ) {
        if !gui {
            pos -= self.position;
        }
        pos *= self.scale;
        pos = Vec2::new(pos.x.floor(), pos.y.floor());
        let size = tile_size.into_f32() * self.scale;
        let size = Vec2::new(size.x.ceil(), size.y.ceil());
        let mut tile_size = tile_size.into_f32();
        tile_size.x /= image.size().x as f32;
        tile_size.y /= image.size().y as f32;
        let mut uvtl = tile.into_f32();
        uvtl.x *= tile_size.x;
        uvtl.y *= tile_size.y;
        let mut uvbr = uvtl + tile_size;
        if flip_h {
            std::mem::swap(&mut uvtl.x, &mut uvbr.x);
        }
        if flip_v {
            std::mem::swap(&mut uvtl.y, &mut uvbr.y);
        }
        self.graphics.draw_rectangle_image_subset_tinted(
            speedy2d::shape::Rectangle::new(pos, pos + size),
            Color::WHITE,
            speedy2d::shape::Rectangle::new(uvtl, uvbr),
            image,
        );
    }

    pub fn draw_autotile(&mut self, screen_size: Vec2, assets: &Assets, layer: &impl AutoLayer) {
        let weather = *watch::interpreter::pywatch::WEATHER.lock().unwrap();
        let (tl, size) = self.view_rect(screen_size, layer.grid_size());
        for (pos, tiles) in layer.autotile_rect(tl, size) {
            for mut tile in tiles {
                if tile.position.x <= 3
                    && tile.position.y <= 1
                    && weather == watch::interpreter::pywatch::Weather::Snowy
                {
                    tile.position.y += 2;
                }
                self.draw_tile(
                    Vec2::new(
                        pos.x as f32 * layer.grid_size().x as f32,
                        pos.y as f32 * layer.grid_size().y as f32,
                    ),
                    false,
                    tile.position,
                    layer.grid_size(),
                    &assets.tileset,
                    tile.flip.horizontal(),
                    tile.flip.vertical(),
                )
            }
        }
    }

    pub fn draw_tiles(&mut self, screen_size: Vec2, assets: &Assets, layer: &impl Tiles) {
        let (tl, size) = self.view_rect(screen_size, layer.grid_size());
        for (pos, tile) in layer.rect(tl, size) {
            if let Some(tile) = tile {
                self.draw_tile(
                    Vec2::new(
                        pos.x as f32 * layer.grid_size().x as f32,
                        pos.y as f32 * layer.grid_size().y as f32,
                    ),
                    false,
                    tile.position,
                    layer.grid_size(),
                    &assets.tileset,
                    tile.flip.horizontal(),
                    tile.flip.vertical(),
                )
            }
        }
    }

    pub fn view_rect(&self, screen_size: Vec2, grid_size: UVec2) -> (IVec2, UVec2) {
        let tl = IVec2::new(
            (self.position.x / grid_size.x as f32).floor() as _,
            (self.position.y / grid_size.y as f32).floor() as _,
        );
        let br = IVec2::new(
            ((self.position.x + screen_size.x) / grid_size.x as f32).ceil() as _,
            ((self.position.y + screen_size.y) / grid_size.y as f32).ceil() as _,
        );
        let size = br - tl;
        (tl, size.into_u32())
    }
}

pub fn get_player_start_position(entities: &world::Entities) -> Vec2 {
    for entity in entities.entities() {
        if matches!(entity.entity, world::Entity::PlayerStartPosition(_)) {
            return entity.position;
        }
    }
    Vec2::ZERO
}

impl<T> world::VectorImpl for Vector2<T>
where
    T: std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + Copy,
{
    type T = T;

    fn new(x: Self::T, y: Self::T) -> Self {
        Self::new(x, y)
    }

    fn x(v: &Self) -> T {
        v.x
    }

    fn y(v: &Self) -> T {
        v.y
    }
}

impl world::ColorImpl for Color {
    fn from_hex(hex: u32) -> Self {
        Self::from_hex_rgb(hex)
    }
}
