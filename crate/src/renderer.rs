use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

use crate::universe::{Cell, Universe};
use crate::utils::WasmUnwrap;


pub struct Renderer {
    universe: Rc<RefCell<Universe>>,
    ctx: CanvasRenderingContext2d,
}

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d, universe: Rc<RefCell<Universe>>) -> Renderer {
        Renderer { universe, ctx }
    }

    pub fn draw(&self) {
        self.draw_cells();
    }

    fn draw_cells(&self) {
        let universe = self.universe.borrow_mut();

        let cells = universe.get_cells();
        let width: u32 = universe.width();
        let height: u32 = universe.height();
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