mod utils;

use js_sys::WebAssembly;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CELL_SIZE: f64 = 5.0; // px
const CELL_SIZEU: u32 = 5; // px

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("game-of-life-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let universe = Universe::new();
    let width = universe.width();
    let height = universe.height();
    canvas.set_height((CELL_SIZEU + 1) * height + 1);
    canvas.set_width((CELL_SIZEU + 1) * width + 1);

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;


    //WebGlRenderingContext
    //CanvasRenderingContext2D
    startRenderLoop(ctx, universe);
    Ok(())
}

fn startRenderLoop(ctx: CanvasRenderingContext2d, universe: Universe) {
    fn request_animation_frame(f: &Closure<FnMut()>) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    let renderer = CanvasRenderer::new(ctx, universe);

    log!("Starting loop...");

    let mut rc = Rc::new(renderer);
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let c = move || {
        if let Some(the_self) = Rc::get_mut(&mut rc) {
            the_self.render();
        };
        request_animation_frame(f.borrow().as_ref().unwrap());
    };

    *g.borrow_mut() = Some(Closure::wrap(Box::new(c) as Box<FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
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
        self.draw_grid();
        self.draw_cells();
    }

    pub fn draw_grid(&self) {
        let width: u32 = self.universe.width();
        let widthf: f64 = f64::from(width);
        let height: u32 = self.universe.height();
        let heightf: f64 = f64::from(height);

        self.ctx.begin_path();
        self.ctx.set_stroke_style(&self.universe.grid_color);

        // Vertical lines.
        for column in 0..width {
            let columnf = f64::from(column);
            self.ctx.move_to(columnf * (CELL_SIZE + 1.0) + 1.0, 0.0);
            self.ctx.line_to(
                columnf * (CELL_SIZE + 1.0) + 1.0,
                (CELL_SIZE + 1.0) * heightf + 1.0,
            );
        }

        // Horizontal lines.
        for row in 0..height {
            let rowf = f64::from(row);
            self.ctx.move_to(0.0, rowf * (CELL_SIZE + 1.0) + 1.0);
            self.ctx.line_to(
                (CELL_SIZE + 1.0) * widthf + 1.0,
                rowf * (CELL_SIZE + 1.0) + 1.0,
            );
        }

        self.ctx.stroke();
    }

    pub fn draw_cells(&self) {
        let cells = self.universe.get_cells();

        let width: u32 = self.universe.width();
        let height: u32 = self.universe.height();
        //let cells = new Uint8Array(memory.buffer, cellsPtr, width * height);

        self.ctx.begin_path();

        // Alive cells.
        self.ctx.set_fill_style(&self.universe.alive_color);

        for row in 0..height {
            let rowf: f64 = f64::from(row);
            for col in 0..width {
                let idx = self.universe.get_index(row, col);
                if cells[idx] != Cell::Alive {
                    continue;
                }

                let colf: f64 = f64::from(col);
                self.ctx.fill_rect(
                    colf * (CELL_SIZE + 1.0) + 1.0,
                    rowf * (CELL_SIZE + 1.0) + 1.0,
                    CELL_SIZE,
                    CELL_SIZE,
                );
            }

        }


        // Dead cells.
        self.ctx.set_fill_style(&self.universe.dead_color);
        for row in 0..height {
            let rowf: f64 = f64::from(row);
            for col in 0..width {
                let idx = self.universe.get_index(row, col);
                if cells[idx] != Cell::Dead {
                    continue;
                }

                let colf: f64 = f64::from(col);
                self.ctx.fill_rect(
                    colf * (CELL_SIZE + 1.0) + 1.0,
                    rowf * (CELL_SIZE + 1.0) + 1.0,
                    CELL_SIZE,
                    CELL_SIZE,
                );
            }
        }


        self.ctx.stroke();
    }
}

#[wasm_bindgen]
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

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    grid_color: wasm_bindgen::JsValue,
    dead_color: wasm_bindgen::JsValue,
    alive_color: wasm_bindgen::JsValue,
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

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 128;
        let height = 128;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        let grid_color: wasm_bindgen::JsValue = JsValue::from_str("#CCCCCC");
        let dead_color: wasm_bindgen::JsValue = JsValue::from_str("#FFFFFF");
        let alive_color: wasm_bindgen::JsValue = JsValue::from_str("#000000");

        Universe {
            width,
            height,
            cells,
            grid_color,
            dead_color,
            alive_color,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                //log!(
                //    "cell[{}, {}] is initially {:?} and has {} live neighbors",
                //   row,
                //    col,
                //    cell,
                //    live_neighbors
                //);

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
                //log!("    it becomes {:?}", next_cell);
                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

use web_sys::console;

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}
