use speedy2d::window::VirtualKeyCode;
use std::collections::HashMap;

#[derive(Default)]
pub struct Controls {
    pub pressed: HashMap<VirtualKeyCode, bool>,
    pub jpressed: HashMap<VirtualKeyCode, bool>,
    pub mods: speedy2d::window::ModifiersState,
}

impl Controls {
    pub fn reset(&mut self) {
        self.jpressed.clear();
    }

    pub fn pressed(&self, virtual_key_code: VirtualKeyCode) -> bool {
        self.pressed
            .get(&virtual_key_code)
            .is_some_and(|pressed| *pressed)
    }

    pub fn jpressed(&self, virtual_key_code: VirtualKeyCode) -> bool {
        self.jpressed
            .get(&virtual_key_code)
            .is_some_and(|pressed| *pressed)
    }

    pub fn left(&self) -> bool {
        self.pressed(VirtualKeyCode::A)
            || self.pressed(VirtualKeyCode::Left)
            || self.pressed(VirtualKeyCode::H)
    }

    pub fn right(&self) -> bool {
        self.pressed(VirtualKeyCode::D)
            || self.pressed(VirtualKeyCode::Right)
            || self.pressed(VirtualKeyCode::L)
    }

    pub fn jump(&self) -> bool {
        self.pressed(VirtualKeyCode::Space)
            || self.pressed(VirtualKeyCode::W)
            || self.pressed(VirtualKeyCode::Up)
            || self.pressed(VirtualKeyCode::K)
    }

    pub fn slide(&self) -> bool {
        self.mods.shift()
            || self.pressed(VirtualKeyCode::S)
            || self.pressed(VirtualKeyCode::Down)
            || self.pressed(VirtualKeyCode::J)
    }

    pub fn watch_toggle(&self) -> bool {
        self.jpressed(VirtualKeyCode::Tab)
    }
}
