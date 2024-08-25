#![vm::pymodule]
use super::vm;
use super::*;
use speedy2d::font::TextLayout;
use std::sync::{Arc, LazyLock, Mutex};
use vm::convert::ToPyObject;
use vm::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    LoadImage(Vec<u8>),
    UnlockNearest,
    LockNearest,
    Run(String),
    AddApp(String),
}

#[derive(Clone, Debug, Default)]
pub struct ActionQueue {
    pub queue: Vec<Action>,
    pub next_image_index: PyImage,
}

pub static ACTION_QUEUE: LazyLock<Arc<Mutex<ActionQueue>>> =
    LazyLock::new(|| Arc::new(Mutex::new(ActionQueue::default())));

pub static IMAGE_SIZE: LazyLock<Arc<Mutex<Vec<Vec2>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

pub static CAPTURE_OUTPUT: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Default)]
pub struct Renderer {
    pub image_map: Vec<speedy2d::image::ImageHandle>,
    pub render_queue: pywatch::RenderQueue,
}

impl Renderer {
    pub fn frame(
        &self,
        vm: &VirtualMachine,
        camera: &mut Camera,
        assets: &Assets,
        screen_space: Rect,
    ) -> PyResult<()> {
        let mut queue = self.render_queue.lock().unwrap();
        for instruction in queue.drain(..) {
            match instruction {
                pywatch::PyRenderInstruction::Image {
                    image,
                    mut position,
                    size,
                    uv,
                } => {
                    let image = self
                        .image_map
                        .get(image)
                        .ok_or_else(|| vm.new_value_error("Invalid image".to_owned()))?;
                    position += screen_space.top_left();
                    position *= camera.scale;
                    let mut size = size.unwrap_or(image.size().into_f32());
                    size *= camera.scale;
                    if let Some(uv) = uv {
                        size.x *= uv.1.x - uv.0.x;
                        size.y *= uv.1.y - uv.0.y;
                    }
                    camera.graphics.draw_rectangle_image_subset_tinted(
                        Rect::new(position, position + size),
                        Color::WHITE,
                        uv.map_or(
                            Rect::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                            |(tl, br)| Rect::new(tl, br),
                        ),
                        image,
                    )
                }
                pywatch::PyRenderInstruction::Text {
                    text,
                    mut position,
                    mut size,
                    color,
                } => {
                    position += screen_space.top_left();
                    position *= camera.scale;
                    size *= camera.scale;
                    camera.graphics.draw_text(
                        position,
                        color,
                        &assets.font.layout_text(
                            &text,
                            size,
                            speedy2d::font::TextOptions::default().with_wrap_to_width(
                                screen_space.width() * camera.scale,
                                speedy2d::font::TextAlignment::Left,
                            ),
                        ),
                    )
                }
            }
        }

        Ok(())
    }
}

impl super::Interpreter {
    pub fn update_queue(
        &mut self,
        graphics: &mut Graphics2D,
        level: &mut world::Level,
        player: &Player,
        apps: &mut Vec<App>,
    ) {
        let queue = std::mem::take(&mut ACTION_QUEUE.lock().unwrap().queue);

        fn range_rect(origin: Vec2, size: Vec2, grid_size: UVec2) -> (IVec2, IVec2) {
            let tl = origin - size / 2.0;
            let tl = IVec2::new(
                (tl.x / grid_size.x as f32).floor() as _,
                (tl.y / grid_size.y as f32).floor() as _,
            );
            let br = origin + size / 2.0;
            let br = IVec2::new(
                (br.x / grid_size.x as f32).ceil() as _,
                (br.y / grid_size.y as f32).ceil() as _,
            );
            (tl, br)
        }

        for action in queue {
            match action {
                Action::LoadImage(image) => {
                    let image = graphics
                        .create_image_from_file_bytes(
                            None,
                            speedy2d::image::ImageSmoothingMode::NearestNeighbor,
                            std::io::Cursor::new(image),
                        )
                        .unwrap();
                    IMAGE_SIZE.lock().unwrap().push(image.size().into_f32());
                    self.renderer.image_map.push(image);
                }
                Action::UnlockNearest => {
                    let origin = player.position + player.size.into_f32() / 2.0;
                    let area = Vec2::new(80.0, 80.0);
                    let (tl, br) = range_rect(origin, area, level.foreground.grid_size());
                    for y in tl.y..br.y {
                        for x in tl.x..br.x {
                            let position = IVec2::new(x, y);
                            if let Some(tile) = level.foreground.get(position) {
                                if tile.position == UVec2::new(7, 3) {
                                    level.foreground.get_mut(position).unwrap().position.x += 1;
                                    level
                                        .foreground
                                        .get_mut(position - IVec2::new_y(1))
                                        .unwrap()
                                        .position
                                        .x += 1;
                                }
                            }
                        }
                    }
                }
                Action::LockNearest => {
                    let origin = player.position + player.size.into_f32() / 2.0;
                    let area = Vec2::new(80.0, 80.0);
                    let (tl, br) = range_rect(origin, area, level.foreground.grid_size());
                    for y in tl.y..br.y {
                        for x in tl.x..br.x {
                            let position = IVec2::new(x, y);
                            if let Some(tile) = level.foreground.get(position) {
                                if tile.position == UVec2::new(8, 3) {
                                    level.foreground.get_mut(position).unwrap().position.x -= 1;
                                    level
                                        .foreground
                                        .get_mut(position - IVec2::new_y(1))
                                        .unwrap()
                                        .position
                                        .x -= 1;
                                }
                            }
                        }
                    }
                }
                Action::Run(code) => {
                    *CAPTURE_OUTPUT.lock().unwrap() = Some(String::new());
                    self.enter(|vm| {
                        vm.run_code_string(self.player_scope.clone(), &code, "<stdin>".to_owned())
                    });
                    let output = CAPTURE_OUTPUT.lock().unwrap().take().unwrap();
                    self.enter(|vm| {
                        if let Some(module) = &self.current_app {
                            let handler = module.get_attr("on_run_output", vm)?;
                            handler.call((output,), vm).map(|_| ())
                        } else {
                            Ok(())
                        }
                    });
                }
                Action::AddApp(module) => {
                    apps.push(App::new(
                        UVec2::new(4, 0),
                        Box::leak(Box::new(module)).as_str(),
                    ));
                }
            }
        }
    }
}

// * Py data types
pub type PyImage = usize;

#[derive(Clone, Debug)]
pub struct PyVec2(Vec2);

impl ToPyObject for PyVec2 {
    fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
        vm.unwrap_pyresult(
            vm.unwrap_pyresult(
                vm.unwrap_pyresult(vm.import("vec", 0))
                    .get_attr("Vector2", vm),
            )
            .call((self.0.x, self.0.y), vm),
        )
    }
}

impl TryFromBorrowedObject<'_> for PyVec2 {
    fn try_from_borrowed_object(vm: &VirtualMachine, obj: &PyObject) -> PyResult<Self> {
        let x = obj.get_attr("x", vm)?.try_float(vm)?.to_f64() as f32;
        let y = obj.get_attr("y", vm)?.try_float(vm)?.to_f64() as f32;
        Ok(Self(Vec2::new(x, y)))
    }
}

// * Frame
#[pyattr]
#[pyclass(module = "watch", name)]
#[derive(Debug, PyPayload)]
pub struct Frame {
    pub controls: Controls,
    pub mouse_pos: Vec2,
    pub render_queue: RenderQueue,
}

#[pyclass]
impl Frame {
    #[pymethod]
    pub fn click(&self) -> bool {
        self.controls.click()
    }

    #[pymethod]
    pub fn mouse_pos(&self) -> PyVec2 {
        PyVec2(self.mouse_pos)
    }

    #[pymethod]
    pub fn typed_text(&self) -> String {
        self.controls.typed_text.to_owned()
    }

    pub fn string_to_vkc(&self, key: String) -> Option<VirtualKeyCode> {
        use speedy2d::window::VirtualKeyCode;
        match key.as_ref() {
            "enter" => Some(VirtualKeyCode::Return),
            "backspace" => Some(VirtualKeyCode::Backspace),
            "left" => Some(VirtualKeyCode::Left),
            "right" => Some(VirtualKeyCode::Right),
            "up" => Some(VirtualKeyCode::Up),
            "down" => Some(VirtualKeyCode::Down),
            "home" => Some(VirtualKeyCode::Home),
            "end" => Some(VirtualKeyCode::End),
            "page_up" => Some(VirtualKeyCode::PageUp),
            "page_down" => Some(VirtualKeyCode::PageDown),
            "escape" => Some(VirtualKeyCode::Escape),
            _ => None,
        }
    }

    #[pymethod]
    pub fn pressed(&self, key: String) -> bool {
        if let Some(vkc) = self.string_to_vkc(key) {
            self.controls.pressed(vkc)
        } else {
            false
        }
    }

    #[pymethod]
    pub fn jpressed(&self, key: String) -> bool {
        if let Some(vkc) = self.string_to_vkc(key) {
            self.controls.jpressed(vkc)
        } else {
            false
        }
    }

    #[pymethod]
    pub fn ctrl(&self) -> bool {
        self.controls.mods.ctrl()
    }

    #[pymethod]
    pub fn shift(&self) -> bool {
        self.controls.mods.shift()
    }

    #[pymethod]
    pub fn alt(&self) -> bool {
        self.controls.mods.alt()
    }

    #[pymethod]
    pub fn draw_image(&self, position: PyVec2, image: PyImage) {
        self.render_queue
            .lock()
            .unwrap()
            .push(PyRenderInstruction::Image {
                image,
                position: position.0,
                size: None,
                uv: None,
            });
    }

    #[pymethod]
    pub fn draw_image_pro(
        &self,
        position: PyVec2,
        image: PyImage,
        size: Option<PyVec2>,
        uv_tl: PyVec2,
        uv_br: PyVec2,
    ) {
        self.render_queue
            .lock()
            .unwrap()
            .push(PyRenderInstruction::Image {
                image,
                position: position.0,
                size: size.map(|size| size.0),
                uv: Some((uv_tl.0, uv_br.0)),
            });
    }

    #[pymethod]
    pub fn draw_text(&self, position: PyVec2, text: String, size: f32, color: u32) {
        self.render_queue
            .lock()
            .unwrap()
            .push(PyRenderInstruction::Text {
                text,
                position: position.0,
                size,
                color: Color::from_hex_rgb(color),
            });
    }
}

// * Weather
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weather {
    #[default]
    Sunny,
    Rainy,
    Snowy,
}

pub static WEATHER: LazyLock<Arc<Mutex<Weather>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Weather::default())));
#[pyfunction]
pub fn set_weather(weather_in: String) {
    let mut weather = WEATHER.lock().unwrap();
    match weather_in.as_str() {
        "sunny" => *weather = Weather::Sunny,
        "rainy" => *weather = Weather::Rainy,
        "snowy" => *weather = Weather::Snowy,
        _ => (),
    };
}

// * Interpreter
#[pyfunction]
pub fn run(code: String) {
    let mut queue = ACTION_QUEUE.lock().unwrap();
    queue.queue.push(Action::Run(code));
}

#[pyfunction]
pub fn print(message: String) {
    if let Some(output) = &mut *CAPTURE_OUTPUT.lock().unwrap() {
        output.push_str(&message);
    } else {
        println!("{}", message);
    }
}

// * MISC
#[pyfunction]
pub fn lock_nearest() {
    let mut queue = ACTION_QUEUE.lock().unwrap();
    queue.queue.push(Action::LockNearest);
}

#[pyfunction]
pub fn unlock_nearest() {
    let mut queue = ACTION_QUEUE.lock().unwrap();
    queue.queue.push(Action::UnlockNearest);
}

#[pyfunction]
pub fn add_app(module: String) {
    let mut queue = ACTION_QUEUE.lock().unwrap();
    queue.queue.push(Action::AddApp(module));
}

#[pyfunction]
pub fn load_image(data: Vec<u8>) -> PyImage {
    let mut queue = ACTION_QUEUE.lock().unwrap();
    queue.queue.push(Action::LoadImage(data));
    queue.next_image_index += 1;
    queue.next_image_index - 1
}

#[pyfunction]
pub fn image_size(image: PyImage) -> PyVec2 {
    PyVec2(IMAGE_SIZE.lock().unwrap()[image])
    // .get(image)
    // .ok_or_else(|| vm.new_value_error("Invalid image".to_owned()))?;
}

#[derive(Clone, Debug)]
pub enum PyRenderInstruction {
    Image {
        image: PyImage,
        position: Vec2,
        size: Option<Vec2>,
        uv: Option<(Vec2, Vec2)>,
    },
    Text {
        text: String,
        position: Vec2,
        size: f32,
        color: Color,
    },
}

pub type RenderQueue = Arc<Mutex<Vec<PyRenderInstruction>>>;
