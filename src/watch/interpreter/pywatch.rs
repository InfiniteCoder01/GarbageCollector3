#![vm::pymodule]
use super::vm;
use super::*;
use speedy2d::font::TextLayout;
use std::sync::{Arc, Mutex};
use vm::convert::ToPyObject;
use vm::*;

#[derive(Clone, Debug, Default)]
pub struct ImageLoadQueue {
    pub queue: Vec<Vec<u8>>,
    pub next_index: PyImage,
}

pub static IMAGE_LOAD_QUEUE: std::sync::LazyLock<Arc<Mutex<ImageLoadQueue>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(ImageLoadQueue::default())));

pub static IMAGE_SIZE: std::sync::LazyLock<Arc<Mutex<Vec<Vec2>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

#[derive(Default)]
pub struct Renderer {
    pub image_map: Vec<speedy2d::image::ImageHandle>,
    pub render_queue: pywatch::RenderQueue,
}

impl Renderer {
    pub fn update(&mut self, graphics: &mut Graphics2D) {
        let mut queue = IMAGE_LOAD_QUEUE.lock().unwrap();
        let mut image_size = IMAGE_SIZE.lock().unwrap();
        for image in queue.queue.drain(..) {
            let image = graphics
                .create_image_from_file_bytes(
                    None,
                    speedy2d::image::ImageSmoothingMode::NearestNeighbor,
                    std::io::Cursor::new(image),
                )
                .unwrap();
            image_size.push(image.size().into_f32());
            self.image_map.push(image);
        }
    }

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

// * Py data types
pub type PyImage = usize;

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
    pub fn icons(&self) -> PyImage {
        0
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

// * MISC
#[pyfunction]
pub fn load_image(data: Vec<u8>) -> PyImage {
    let mut queue = IMAGE_LOAD_QUEUE.lock().unwrap();
    queue.queue.push(data);
    queue.next_index += 1;
    queue.next_index - 1
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
