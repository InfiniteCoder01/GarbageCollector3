#![feature(proc_macro_hygiene)]
#![feature(custom_inner_attributes)]

use assets::Assets;
use controls::Controls;
use player::Player;
use speedy2d::color::Color;
use speedy2d::dimen::*;
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::Graphics2D;
use watch::Watch;
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
        let window = speedy2d::Window::new_centered("Speedy2D: Animation", (800, 800)).unwrap();
        window.run_loop(handler)
    }
}

struct GarbageCollector3 {
    stopwatch: speedy2d::time::Stopwatch,
    assets: Option<Assets>,
    camera: Vec2,
    controls: Controls,

    level_index: usize,
    world: world::World,
    player: Player,
    watch: Watch,
}

impl GarbageCollector3 {
    fn new() -> Self {
        let world = world::World::load();
        let player = Player::new(get_player_start_position(&world.level_0.marks));
        Self {
            stopwatch: speedy2d::time::Stopwatch::new().unwrap(),
            assets: None,
            camera: Vec2::ZERO,
            controls: Controls::default(),

            level_index: 0,
            world,
            player,
            watch: Watch::default(),
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
        let level = &self.world[self.level_index];

        let scale = helper.get_size_pixels().y as f32 / 256.0;
        let screen_size = helper.get_size_pixels().into_f32() / scale;
        if !self.watch.open {
            self.player.update(delta_time, level, &self.controls);
            for mark in level.marks.entities() {
                if matches!(mark.entity, world::Entity::EndOfTheLevel(_))
                    && self.player.overlaps(mark)
                {
                    self.level_index += 1;
                    self.player.position =
                        get_player_start_position(&self.world[self.level_index].marks);
                    self.camera = self.player.position + self.player.size.into_f32() / 2.0
                        - screen_size / 2.0;
                }
            }
        }

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

        camera.graphics.clear_screen(level.bg_color);
        camera.draw_tiles(screen_size, assets, &level.background);
        camera.draw_autotile(screen_size, assets, &level.solid);
        camera.draw_tiles(screen_size, assets, &level.ambient_decorations);
        self.player.draw(&mut camera, assets);
        self.watch
            .draw(helper, delta_time, &self.controls, &mut camera, assets);
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
        let (tl, size) = self.view_rect(screen_size, layer.grid_size());
        for (pos, tiles) in layer.autotile_rect(tl, size) {
            for tile in tiles {
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

pub fn get_player_start_position(marks: &world::Marks) -> Vec2 {
    for mark in marks.entities() {
        if matches!(mark.entity, world::Entity::PlayerStartPosition(_)) {
            return mark.position;
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
