use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils::WasmUnwrap;

pub struct Game {
    callback: Box<FnMut()>,
}

pub struct RequestAnimationFrameLoop {
    instance: Rc<Game>,
}

impl RequestAnimationFrameLoop {
    pub fn new(instance: Rc<Game>) -> RequestAnimationFrameLoop {
        RequestAnimationFrameLoop { instance }
    }

    pub fn start(&mut self) {
        let mut animationloop = self.instance.clone();
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        fn request_animation_frame(f: &Closure<FnMut()>) {
            web_sys::window()
                .unwrap_wasm()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .unwrap_wasm();
        }
        let c = move || {
            if let Some(the_self) = Rc::get_mut(&mut animationloop) {
                (the_self.callback)();
            }
            request_animation_frame(f.try_borrow().unwrap_wasm().as_ref().unwrap_wasm());
        };

        *g.try_borrow_mut().unwrap_wasm() = Some(Closure::wrap(Box::new(c) as Box<FnMut()>));
        request_animation_frame(g.try_borrow().unwrap_wasm().as_ref().unwrap_wasm());
    }
}

impl Game {
    pub fn new(callback: Box<FnMut()>) -> Game {
        Game { callback }
    }
}
