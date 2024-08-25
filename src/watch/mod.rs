use super::*;
use speedy2d::shape::Rect;

pub mod interpreter;
pub use interpreter::Interpreter;

const APP_SIZE: u32 = 24;

pub struct Watch {
    pub open: bool,
    pub apps: Vec<App>,

    pub interpreter: Interpreter,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            open: false,
            apps: vec![
                App::new(UVec2::new(0, 0), "weather"),
                App::new(UVec2::new(1, 0), "keyring"),
                App::new(UVec2::new(2, 0), "placeholder"),
                App::new(UVec2::new(3, 0), "terminal"),
            ],
            interpreter: Interpreter::default(),
        }
    }
}

impl Watch {
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        helper: &mut WindowHelper,
        delta_time: f32,
        controls: &Controls,
        camera: &mut Camera,
        assets: &Assets,
        level: &mut world::Level,
        player: &Player,
    ) {
        {
            let weather = *interpreter::pywatch::WEATHER.lock().unwrap();
            let (from, to) = if weather == interpreter::pywatch::Weather::Snowy {
                (UVec2::new(7, 0), UVec2::new(7, 1))
            } else {
                (UVec2::new(7, 1), UVec2::new(7, 0))
            };
            for tile in level.background.tiles.iter_mut().flatten() {
                if tile.position == from {
                    tile.position = to;
                }
            }
        }

        self.interpreter
            .update(delta_time, camera.graphics, level, player, &mut self.apps);
        if controls.watch_toggle() {
            self.open = !self.open;
        }
        if !self.open {
            return;
        }

        let center = helper.get_size_pixels().into_f32() / 2.0;
        let size = assets.watch.image.size().into_f32() * camera.scale;
        camera.graphics.draw_rectangle_image(
            Rect::new(center - size / 2.0, center + size / 2.0),
            &assets.watch.image,
        );

        let center = center / camera.scale;
        let size = Vec2::new(128.0, 128.0);
        let screen_space = Rect::new(center - size / 2.0, center + size / 2.0);
        let mouse_pos = controls.mouse_pos / camera.scale;

        if self.interpreter.current_app.is_some() {
            let mouse_pos = mouse_pos - screen_space.top_left();
            let frame = interpreter::pywatch::Frame {
                controls: controls.clone(),
                mouse_pos,
                render_queue: self.interpreter.renderer.render_queue.clone(),
            };
            self.interpreter.frame(camera, assets, screen_space, frame);
        } else {
            let mut cursor = *screen_space.top_left();
            let padding = 4.0; // (screen_space.width() - APP_SIZE as f32 * 4.0) / 3.0;
            for app in &self.apps {
                if Rect::new(cursor, cursor + Vec2::new(APP_SIZE as f32, APP_SIZE as f32))
                    .contains(mouse_pos)
                    && controls.click()
                {
                    self.interpreter.current_app = self.interpreter.enter(|vm| {
                        if let Ok(module) = self
                            .interpreter
                            .player_scope
                            .globals
                            .get_item(app.module, vm)
                        {
                            Ok(module)
                        } else {
                            vm.import(app.module, 0)
                        }
                    });
                }
                app.draw(cursor, camera, assets);
                cursor.x += APP_SIZE as f32 + padding;
                if cursor.x + APP_SIZE as f32 > screen_space.right() {
                    cursor.x = screen_space.left();
                    cursor.y += APP_SIZE as f32 + padding;
                }
            }
        }
    }
}

pub struct App {
    icon: UVec2,
    module: &'static str,
}

impl App {
    pub fn new(icon: UVec2, module: &'static str) -> Self {
        Self { icon, module }
    }

    pub fn draw(&self, position: Vec2, camera: &mut Camera, assets: &Assets) {
        camera.draw_tile(
            position,
            true,
            self.icon,
            UVec2::new(APP_SIZE, APP_SIZE),
            &assets.watch.icons,
            false,
            false,
        )
    }
}
