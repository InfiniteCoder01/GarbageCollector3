use super::*;

pub struct Watch {
    pub open: bool,
}

impl Watch {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn update(&mut self, _delta_time: f32, controls: &Controls) {
        if controls.watch_toggle() {
            self.open = !self.open;
        }
        if !self.open {
            return;
        }
    }

    pub fn draw(&self, helper: &mut WindowHelper, camera: &mut Camera, assets: &Assets) {
        if !self.open {
            return;
        }
        let center = helper.get_size_pixels().into_f32() / 2.0;
        let size = assets.watch.image.size().into_f32() * camera.scale;
        camera.graphics.draw_rectangle_image(
            speedy2d::shape::Rect::new(center - size / 2.0, center + size / 2.0),
            &assets.watch.image,
        );
    }
}
