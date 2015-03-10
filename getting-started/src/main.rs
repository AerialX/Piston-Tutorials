#![feature(libc)]

extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate libc;

use std::cell::RefCell;
use piston::window::WindowSettings;
use piston::event::{
    RenderArgs,
    RenderEvent,
    UpdateArgs,
    UpdateEvent
};
use graphics::{
    Context,
    rectangle,
    RelativeTransform
};
use sdl2_window::Sdl2Window as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

pub struct App {
    gl: GlGraphics,       // OpenGL drawing backend.
    rotation: f64 // Rotation for the square.
}

impl App {
    fn render(&mut self, _: &mut Window, args: &RenderArgs) {
        const GREEN:  [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:    [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        // Set up a context to draw into.
        let context = &Context::abs(args.width as f64, args.height as f64);

        let center_context = &context.trans((args.width / 2) as f64, (args.height / 2) as f64)
                                     .rot_rad(self.rotation)
                                     .trans(-25.0, -25.0);
        let square = rectangle::square(0.0, 0.0, 50.0);

        self.gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
            // Clear the screen.
            graphics::clear(GREEN, gl);
            // Draw a box rotating around the middle of the screen.
            graphics::rectangle(RED, square, center_context, gl);
        });
    }

    fn update(&mut self, _: &mut Window, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

fn main() {
    let opengl = OpenGL::WebGL_1_0;
    // Create an SDL window.
    let window = Window::new(
        opengl,
        WindowSettings::default(),
    );
    let window = RefCell::new(window);

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
    };

    emscripten_main_loop(move || {
        app.update(&mut window.borrow_mut(), &UpdateArgs { dt: 1.0 / 30.0 });
        app.render(&mut window.borrow_mut(), &RenderArgs { ext_dt: 0.0, width: 640, height: 480 });
    }, 30, false);

    // Make sure these stay linked in because yay LLVM intrinsics
    llvm_sin_f64(0.0);
    llvm_cos_f64(0.0);
}

thread_local!(static MAIN_LOOP: RefCell<Option<Box<FnMut()>>> = RefCell::new(None));
fn emscripten_main_loop<F: FnMut() + 'static>(f: F, fps: usize, simulate_infinite_loop: bool) {
    MAIN_LOOP.with(|v| *v.borrow_mut() = Some(Box::new(f) as Box<FnMut()>));
    unsafe {
        emscripten_set_main_loop(emscripten_main_loop_, fps as libc::c_int, simulate_infinite_loop);
    }
}

extern "C" fn emscripten_main_loop_() {
    MAIN_LOOP.with(|v| if let Some(f) = v.borrow_mut().as_mut() {
        f();
    });
}

extern {
    fn sin(v: f64) -> f64;
    fn cos(v: f64) -> f64;

    fn emscripten_set_main_loop(f: extern "C" fn(), fps: libc::c_int, simulate_infinite_loop: bool);
}

#[no_mangle]
#[inline(never)]
pub extern fn llvm_sin_f64(v: f64) -> f64 {
    unsafe {
        sin(v)
    }
}

#[no_mangle]
#[inline(never)]
pub extern fn llvm_cos_f64(v: f64) -> f64 {
    unsafe {
        cos(v)
    }
}
