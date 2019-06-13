mod utils;

use std::cell::RefCell;
use std::process;
use std::rc::Rc;
use std::usize;


use wasm_bindgen::prelude::*;

use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        #[cfg(feature = "console_error_panic_hook")]
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

impl<T> WasmUnwrap<T> for Option<T> {
    fn unwrap_wasm(self) -> T {
        match self {
            Some(w) => w,
            None => process::abort(),
        }
    }
}

impl<T, E> WasmUnwrap<T> for Result<T, E> {
    fn unwrap_wasm(self) -> T {
        match self {
            Ok(w) => w,
            Err(e) => process::abort(),
        }
    }
}

trait WasmUnwrap<T> {
    fn unwrap_wasm(self) -> T;
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();

    let window = web_sys::window().unwrap_wasm();
    let document = window.document().unwrap_wasm();

    let canvas = document
        .get_element_by_id("game-of-life-canvas")
        .unwrap_wasm()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap_wasm();

    let universe = Universe::new();
    let width = universe.width();
    let height = universe.height();
    canvas.set_height(height);
    canvas.set_width(width);

    let options = JsValue::from_str("{ alpha: false }");
    let ctx = canvas
        .get_context_with_context_options("2d", &options)
        .unwrap_wasm()
        .unwrap_wasm()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap_wasm();

    start_render_loop(window, ctx, universe);
    Ok(())
}

fn start_render_loop(window: web_sys::Window, ctx: CanvasRenderingContext2d, universe: Universe) {
    fn request_animation_frame(f: &Closure<FnMut()>) {
        web_sys::window()
            .unwrap_wasm()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    ctx.set_image_smoothing_enabled(false);
    let renderer = CanvasRenderer::new(ctx, universe);

    log!("Starting loop...");

    let mut rc = Rc::new(renderer);
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let performance = window
        .performance()
        .expect("performance should be available");
    let mut lastFrameTimeStamp = performance.now();
    let c = move || {
        if let Some(the_self) = Rc::get_mut(&mut rc) {
            the_self.render();
        };

        let now = performance.now();
        let delta = now - lastFrameTimeStamp;
        lastFrameTimeStamp = now;
        let fps = 1.0 / delta * 1000.0;
        log!("fps: {}", fps);

        request_animation_frame(f.borrow().as_ref().unwrap_wasm());
    };

    *g.borrow_mut() = Some(Closure::wrap(Box::new(c) as Box<FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap_wasm());
}

pub struct CanvasRenderer {
    universe: Universe,
    ctx: CanvasRenderingContext2d,
}

impl CanvasRenderer {
    pub fn new(ctx: CanvasRenderingContext2d, universe: Universe) -> CanvasRenderer {
        CanvasRenderer { universe, ctx }
    }

    pub fn render(&mut self) {
        self.universe.tick();
        self.draw_cells();
    }

    pub fn draw_cells(&mut self) {
        let cells = self.universe.get_cells();
        let width: u32 = self.universe.width();
        let height: u32 = self.universe.height();
        let length = (width * height) as usize;


        let mut data = vec![0; length * 4];

        let image_data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data), width, height)
                .unwrap_wasm();

        for i in 0..length {
            if cells[i] == Cell::Alive {
                let idx = i * 4;
                data[idx] = 0;
                data[idx + 1] = 0;
                data[idx + 2] = 0;
                data[idx + 3] = 255;
            } else if cells[i] == Cell::Dead {
                let idx = i * 4;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
        self.ctx.put_image_data(&image_data, 0.0, 0.0).unwrap_wasm();
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;

        let n = self.get_index(north, column);
        count += self.cells[n] as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;

        let w = self.get_index(row, west);
        count += self.cells[w] as u8;

        let e = self.get_index(row, east);
        count += self.cells[e] as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;

        let s = self.get_index(south, column);
        count += self.cells[s] as u8;

        let se = self.get_index(south, east);
        count += self.cells[se] as u8;

        count
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn new() -> Universe {
        let width = 1024;
        let height = 768;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };
                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }
}