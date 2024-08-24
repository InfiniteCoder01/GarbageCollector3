use super::*;
use rustpython_vm as vm;

pub mod pywatch;

pub struct Interpreter {
    pub interpreter: vm::Interpreter,
    pub initialized: bool,

    pub renderer: pywatch::Renderer,
    pub current_app: Option<rustpython_vm::PyObjectRef>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            interpreter: rustpython::InterpreterConfig::new()
                .init_stdlib()
                .add_native_module("watch".to_owned(), pywatch::make_module)
                .interpreter(),
            initialized: false,

            renderer: pywatch::Renderer::default(),
            current_app: None,
        }
    }
}

impl Interpreter {
    pub fn update(&mut self, graphics: &mut Graphics2D) {
        if !self.initialized {
            self.enter(|vm| {
                macro_rules! import {
                    ($($name:ident = $path:literal)*) => {
                        $(vm::import::import_codeobj(
                            vm,
                            stringify!($name),
                            vm.ctx.new_code(vm::py_compile!(file = $path)),
                            true,
                        )?;)*
                    };
                }
                import! {
                    vec = "apps/vec.py"
                    ui = "apps/ui.py"
                    placeholder = "apps/placeholder.py"

                    weather = "apps/weather.py"
                }
                Ok(())
            });
            self.initialized = true;
        }
        self.renderer.update(graphics);
    }

    pub fn frame(
        &mut self,
        camera: &mut Camera,
        assets: &Assets,
        screen_space: Rect,
        frame: pywatch::Frame,
    ) {
        if let Some(module) = &self.current_app {
            let result = self.enter(|vm| {
                let frame_fn = module.get_attr("frame", vm)?;
                let result = frame_fn.call((frame,), vm)?.try_to_bool(vm);
                self.renderer.frame(vm, camera, assets, screen_space)?;
                result
            });
            if result != Some(true) {
                self.current_app = None;
            }
        }
    }

    pub fn enter<R>(
        &self,
        callback: impl FnOnce(&rustpython_vm::VirtualMachine) -> rustpython_vm::PyResult<R>,
    ) -> Option<R> {
        self.interpreter.enter(|vm| match callback(vm) {
            Ok(val) => Some(val),
            Err(err) => {
                vm.print_exception(err);
                None
            }
        })
    }
}
