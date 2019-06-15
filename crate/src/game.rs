use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils::WasmUnwrap;

pub struct AnimationCallback {
    callback: Box<FnMut()>,
}

pub struct RequestAnimationFrameLoop {
    callback: Rc<AnimationCallback>,
}

impl RequestAnimationFrameLoop {
    pub fn new(callback: Rc<AnimationCallback>) -> RequestAnimationFrameLoop {
        RequestAnimationFrameLoop { callback }
    }

    pub fn start(&mut self) {
        let mut callback = self.callback.clone();
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        fn request_animation_frame(f: &Closure<FnMut()>) {
            web_sys::window()
                .unwrap_wasm()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .unwrap_wasm();
        }
        let c = move || {
            if let Some(r) = Rc::get_mut(&mut callback) {
                (r.callback)();
            }
            request_animation_frame(f.try_borrow().unwrap_wasm().as_ref().unwrap_wasm());
        };

        *g.try_borrow_mut().unwrap_wasm() = Some(Closure::wrap(Box::new(c) as Box<FnMut()>));
        request_animation_frame(g.try_borrow().unwrap_wasm().as_ref().unwrap_wasm());
    }
}

impl AnimationCallback {
    pub fn new(callback: Box<FnMut()>) -> AnimationCallback {
        AnimationCallback { callback }
    }
}
