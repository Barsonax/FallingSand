#[macro_use]
mod utils;

mod game;
mod renderer;
mod universe;

use crate::game::{AnimationCallback, RequestAnimationFrameLoop};
use crate::renderer::Renderer;
use crate::universe::Universe;
use std::cell::RefCell;
use std::rc::Rc;
use utils::WasmUnwrap;

use wasm_bindgen::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

extern crate web_sys;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    log!("main");

    let window = web_sys::window().unwrap_wasm();
    let document = window.document().unwrap_wasm();
    let body = document.body().unwrap_wasm();

    let game_of_life_canvas_element = document.create_element("canvas").unwrap_wasm();
    game_of_life_canvas_element.set_id("game-of-life-canvas");
    game_of_life_canvas_element
        .set_attribute("style", "transform: scaleY(1) scaleX(1)")
        .unwrap_wasm();
    body.insert_before(&game_of_life_canvas_element, None)
        .unwrap_wasm();

    let canvas = document
        .get_element_by_id("game-of-life-canvas")
        .unwrap_wasm()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap_wasm();

    let fps_div = document.create_element("div").unwrap_wasm();
    fps_div.set_id("fps");
    body.insert_before(&fps_div, None).unwrap_wasm();

    let width = 1024;
    let height = 768;

    let universe_ptr = Rc::new(RefCell::new(Universe::new(width, height)));
    canvas.set_width(width);
    canvas.set_height(height);

    let ctx = canvas
        .get_context("2d")
        .unwrap_wasm()
        .unwrap_wasm()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap_wasm();

    let renderer_ptr = Rc::new(RefCell::new(Renderer::new(ctx, universe_ptr.clone())));

    let callback = Rc::new(AnimationCallback::new(Box::new(move || {
        universe_ptr.borrow_mut().tick();
        renderer_ptr.borrow_mut().draw();
    })));
    let mut animation_loop = RequestAnimationFrameLoop::new(callback);

    log!("start animation loop");
    animation_loop.start();

    Ok(())
}
