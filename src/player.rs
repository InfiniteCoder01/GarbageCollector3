use super::*;

const FRAMES: &[(&str, u32)] = &[
    ("idle", 1),
    ("run", 8),
    ("jump", 1),
    ("fall", 1),
    ("land", 2),
    ("wall_slide", 1),
    ("kick", 1),
    ("slide_start", 1),
    ("slide", 1),
    ("slide_end", 1),
];

pub struct Player {
    pub frames: std::collections::HashMap<&'static str, std::ops::Range<u32>>,
    pub frame_count: u32,

    pub velocity: Vec2,
    pub position: Vec2,
    pub grounded: bool,
    pub slide_timeout: f32,
    pub slippery: bool,

    pub last_grounded: f32,
    pub flip: bool,
    pub frame: f32,
    pub animation: &'static str,
    pub looped: bool,

    pub size: UVec2,
}

impl Player {
    pub fn new(position: Vec2) -> Self {
        let mut frames = std::collections::HashMap::new();
        let mut frame_count = 0;
        for (name, length) in FRAMES {
            frames.insert(*name, frame_count..frame_count + length);
            frame_count += length;
        }
        Self {
            frames,
            frame_count,

            velocity: Vec2::ZERO,
            position,
            grounded: false,
            slide_timeout: 0.0,
            slippery: false,

            last_grounded: 1.0,
            flip: false,
            frame: 0.0,
            animation: "idle",
            looped: true,

            size: UVec2::ZERO,
        }
    }

    pub fn update(&mut self, delta_time: f32, level: &world::Level, controls: &Controls) {
        let key_dir = controls.right() as i32 - controls.left() as i32;
        let target_velocity = key_dir as f32 * 196.0;

        if self.slide_timeout > 0.0 {
            self.slide_timeout -= delta_time;
        }
        if self.grounded && self.animation != "slide" && self.animation != "slide_start" {
            let blend = 1.0 - if self.slippery { 0.3_f32 } else { 0.003_f32 }.powf(delta_time);
            self.velocity.x += (target_velocity - self.velocity.x) * blend;
            if controls.slide() && self.velocity.x.abs() > 180.0 && self.slide_timeout <= 0.0 {
                let anim = self.animation;
                self.animation = "slide";
                if self.collides(level) {
                    self.animation = anim;
                } else {
                    self.transition("slide_start");
                }
            }
        } else if self.animation == "slide" || self.animation == "slide_start" {
            self.velocity.x += self.velocity.x * (0.6_f32.powf(delta_time) - 1.0);
            if !controls.slide() || !self.grounded || self.velocity.x.abs() < 128.0 {
                let anim = self.animation;
                self.animation = "idle";
                if self.collides(level) {
                    self.animation = anim;
                    self.velocity.x = self.velocity.x.signum() * 128.0;
                } else {
                    self.transition("slide_end");
                    if controls.slide() {
                        self.slide_timeout = 0.6;
                    }
                }
            }
        }

        self.velocity.y += 18.0 * 32.0 * delta_time;
        if self.animation == "wall_slide" {
            self.velocity.y = self.velocity.y.min(48.0);
        }
        if (self.grounded || self.animation == "wall_slide")
            && self.animation != "slide"
            && self.animation != "slide_start"
            && controls.jump()
        {
            if self.animation == "wall_slide" {
                self.velocity.x = self.velocity.x.signum() * -64.0;
                self.flip = self.velocity.x < 0.0;
                self.velocity.y = -256.0;
                self.transition("kick");
            } else {
                self.velocity.y = -200.0;
                self.transition("jump");
            }
        }

        self.last_grounded += delta_time;

        let motion = self.velocity * delta_time;
        if self.grounded {
            self.last_grounded = 0.0;
        }
        self.grounded = false;
        if self.animation == "wall_slide" {
            self.animation = "idle";
        }
        self.slippery = false;
        self.move_in_steps(level, Vec2::new_x(motion.x));
        self.move_in_steps(level, Vec2::new_y(motion.y));
        while self.collides(level) {
            self.position.y -= 0.1;
        }

        if key_dir != 0 {
            if self.animation == "kick" {
                if (self.velocity.x < 0.0) != self.flip {
                    self.transition("fall");
                }
                self.flip = self.velocity.x < 0.0;
            } else if self.animation == "wall_slide" || self.animation.starts_with("slide") {
                self.flip = self.velocity.x < 0.0;
            } else {
                self.flip = key_dir < 0;
            }
        }

        let mut animation_speed = 7.0;
        if self.animation == "land" {
            // Wait for land animation to end
        } else if self.animation == "wall_slide" {
            // *Sliding*
        } else if self.animation.starts_with("slide") {
            if self.animation.contains('_') {
                animation_speed = 9.0;
            }
        } else if !self.grounded {
            if self.velocity.y > 0.0 {
                self.transition("fall");
            }
        } else if self.velocity.x.abs() > 70.0 || (key_dir != 0 && self.velocity.x.abs() > 10.0) {
            self.transition("run");
            animation_speed = self.velocity.x.abs() * 0.06;
        } else if self.animation != "run" || (self.frame as u32) & 0b010 == 0 {
            self.transition("idle");
        }
        self.frame += delta_time * animation_speed;
        let frame_count = self.frames[self.animation].clone().count() as f32;
        if self.frame >= frame_count {
            if self.animation == "land" {
                self.transition("idle");
            }
            if self.animation == "slide_start" {
                self.transition("slide");
            }
            if self.animation == "slide_end" {
                self.transition("idle");
            }
            if self.looped {
                self.frame -= frame_count;
            } else {
                self.frame = frame_count - 1.0;
            }
        }
    }

    pub fn collides(&mut self, level: &world::Level) -> bool {
        let (tl, size) = self.rect(level.solid.grid_size());
        let br = tl + size.into_i32();
        if tl.x < 0
            || tl.y < 0
            || br.x > level.solid.size.x as i32
            || br.y > level.solid.size.y as i32
        {
            return true;
        }
        for (pos, tile) in level.solid.rect(tl, size) {
            match tile {
                Some(world::SolidTile::Ground) => return true,
                Some(world::SolidTile::Lamp) => {
                    if level.solid.get(pos - IVec2::new_y(1)) != Some(&world::SolidTile::Lamp) {
                        return true;
                    }
                }
                _ => (),
            }
        }
        for tile in level.background.rect(tl, size).flat_map(|(_, tile)| tile) {
            if tile.position == UVec2::new(7, 1) {
                self.slippery = true;
                return true;
            }
        }
        false
    }

    pub fn transition(&mut self, animation: &'static str) {
        if self.animation == animation {
            return;
        }
        self.animation = animation;
        self.looped = !matches!(
            animation,
            "jump" | "land" | "kick" | "slide_start" | "slide_end"
        );
        self.frame = 0.0;
    }

    pub fn move_in_steps(&mut self, level: &world::Level, motion: Vec2) {
        let mut distance = motion.magnitude();
        if distance == 0.0 {
            return;
        }
        let mut step = 0.1;
        let dir = motion / distance;
        while distance > 0.0 {
            if distance < step {
                step = distance;
            }
            distance -= step;
            self.position += dir * step;
            if self.collides(level) {
                self.position -= dir * step;
                if motion.x != 0.0 {
                    if self.last_grounded > 0.3 {
                        self.transition("wall_slide");
                    } else if self.animation == "slide" || self.animation == "slide_start" {
                        self.transition("slide_end");
                    }
                }
                if motion.y != 0.0 {
                    self.velocity.y = 0.0;
                }
                if motion.y > 0.0 {
                    self.grounded = true;
                    if self.last_grounded > 0.1
                        && self.animation != "slide"
                        && self.animation != "slide_start"
                    {
                        self.transition("land");
                    }
                }
                break;
            }
        }
    }

    pub fn rect(&self, grid_size: UVec2) -> (IVec2, UVec2) {
        let (x_range, y_range) = if self.animation == "slide" || self.animation == "slide_start" {
            ((0.3, 1.0), (0.8, 1.0))
        } else {
            ((0.4, 0.6), (0.0, 1.0))
        };
        let tl = IVec2::new(
            ((self.position.x + self.size.x as f32 * x_range.0) / grid_size.x as f32).floor() as _,
            ((self.position.y + self.size.y as f32 * y_range.0) / grid_size.y as f32).floor() as _,
        );
        let br = IVec2::new(
            ((self.position.x + self.size.x as f32 * x_range.1) / grid_size.x as f32).ceil() as _,
            ((self.position.y + self.size.y as f32 * y_range.1) / grid_size.y as f32).ceil() as _,
        );
        let size = br - tl;
        (tl, size.into_u32())
    }

    pub fn draw(&self, camera: &mut Camera, assets: &Assets) {
        camera.draw_tile(
            self.position,
            false,
            UVec2::new_x(self.frame as u32 + self.frames[self.animation].start),
            self.size,
            &assets.player.image,
            self.flip,
            false,
        )
    }

    pub fn overlaps(&self, entity: &world::EntityObject) -> bool {
        self.position.x + self.size.x as f32 > entity.top_left().x
            && self.position.y + self.size.y as f32 > entity.top_left().y
            && self.position.x < entity.top_left().x + entity.size.x as f32
            && self.position.y < entity.top_left().y + entity.size.y as f32
    }
}
