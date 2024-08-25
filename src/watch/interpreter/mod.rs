use super::*;
use rustpython_vm as vm;
use vm::scope::Scope;

pub mod pywatch;

pub struct Interpreter {
    pub interpreter: vm::Interpreter,
    pub initialized: bool,

    pub renderer: pywatch::Renderer,
    pub current_app: Option<rustpython_vm::PyObjectRef>,
    pub player_scope: Scope,
}

impl Default for Interpreter {
    fn default() -> Self {
        let interpreter = rustpython::InterpreterConfig::new()
            .init_stdlib()
            .add_native_module("watch".to_owned(), pywatch::make_module)
            .interpreter();
        let player_scope = interpreter.enter(|vm| vm.new_scope_with_builtins());
        interpreter.enter(|vm| {
            if let Err(err) = vm.run_code_string(
                player_scope.clone(),
                "import watch\ndef print(*args): watch.print(' '.join(str(arg) for arg in args))",
                "print_hook.py".to_owned(),
            ) {
                vm.print_exception(err);
            }
        });
        Self {
            interpreter,
            initialized: false,

            renderer: pywatch::Renderer::default(),
            current_app: None,
            player_scope,
        }
    }
}

impl Interpreter {
    pub fn update(
        &mut self,
        delta_time: f32,
        graphics: &mut Graphics2D,
        level: &mut world::Level,
        player: &Player,
    ) {
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
                    keyring = "apps/keyring.py"
                    terminal = "apps/terminal.py"
                }
                Ok(())
            });
            self.initialized = true;
        }
        self.update_queue(graphics, level, player);

        for entity in level.entities.entities_mut() {
            if let world::Entity::Platform(platform) = &mut entity.entity {
                let result = self.interpreter.enter(|vm| {
                    vm.compile(
                        &platform.condition,
                        vm::compiler::Mode::Single,
                        "world.ldtk".to_owned(),
                    )
                    .ok()
                    .and_then(|code_obj| vm.run_code_obj(code_obj, self.player_scope.clone()).ok())
                    .and_then(|result| result.try_to_bool(vm).ok())
                    .is_some_and(|result| result)
                });
                let target = if result {
                    platform.point_true
                } else {
                    platform.point_false
                };
                let target = target.into_f32() * world::Entities::GRID_SIZE as f32;
                if (target - entity.position).magnitude_squared() > 2.0 {
                    if let Some(dir) = (target - entity.position).normalize() {
                        entity.position += dir * 10.0 * delta_time;
                    }
                }
            }
        }
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
                if let Some(output) = &mut *pywatch::CAPTURE_OUTPUT.lock().unwrap() {
                    vm.write_exception(&mut *output, &err).unwrap();
                    return None;
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let mut out = String::new();
                    vm.write_exception(&mut out, &err).unwrap();
                    web_sys::console::error_1(&out.into());
                }
                #[cfg(not(target_arch = "wasm32"))]
                vm.print_exception(err);
                None
            }
        })
    }
}
