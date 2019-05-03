use crate::Universe;
use wasm_bindgen::{prelude::*, JsCast, JsValue};

use web_sys::{CanvasRenderingContext2d, HtmlBodyElement, HtmlCanvasElement};

const CELL_SIZE: u32 = 5; // px
const GRID_COLOR: &'static str = "#222";
const DEAD_COLOR: &'static str = "#333";
const ALIVE_COLOR: &'static str = "#DDD";

pub struct Game {
    universe: Universe,
    context: CanvasRenderingContext2d,
}

impl Game {
    pub fn new() -> Game {
        let width = 64;
        let height = 64;
        let universe = Universe::new(width, height);

        let document = web_sys::window().unwrap().document().unwrap();
        document.set_title("Conway's Game of Life!");

        let body = document
            .body()
            .unwrap();
        body.style().set_css_text("background: black");

        let body = body.dyn_into::<HtmlBodyElement>()
            .unwrap();
        

        let canvas = document.create_element("canvas").unwrap();
        body.append_child(&canvas).unwrap();
        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        canvas.set_width((CELL_SIZE + 1) * width + 1);
        canvas.set_height((CELL_SIZE + 1) * height + 1);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Game { universe, context }
    }
    pub fn update(&mut self) {
        self.draw_grid();
        self.draw_cells();
        self.universe.tick();
    }
    fn draw_cells(&mut self) {
        self.context.begin_path();

        for row in 0..self.universe.height() {
            for col in 0..self.universe.width() {
                let alive = self.universe.is_alive(row, col);
                let alive_color = JsValue::from_str(&ALIVE_COLOR);
                let dead_color = JsValue::from_str(&DEAD_COLOR);
                self.context
                    .set_fill_style(if alive { &alive_color } else { &dead_color });
                self.context.fill_rect(
                    (col * (CELL_SIZE + 1) + 1) as f64,
                    (row * (CELL_SIZE + 1) + 1) as f64,
                    CELL_SIZE as f64,
                    CELL_SIZE as f64,
                );
            }
        }
        self.context.stroke();
    }
    fn draw_grid(&self) {
        self.context.begin_path();
        self.context.set_stroke_style(&JsValue::from_str(GRID_COLOR));

        for i in 0..self.universe.width() {
            self.context.move_to((i * (CELL_SIZE + 1) + 1) as f64, 0.);
            self.context.line_to(
                (i * (CELL_SIZE + 1) + 1) as f64,
                ((CELL_SIZE + 1) * self.universe.height() + 1) as f64,
            );
        }

        for i in 0..self.universe.height() {
            self.context.move_to(0., (i * (CELL_SIZE + 1) + 1) as f64);
            self.context.line_to(
                ((CELL_SIZE + 1) * self.universe.width() + 1) as f64,
                (i * (CELL_SIZE + 1) + 1) as f64,
            );
        }
        self.context.stroke();
    }
}

pub struct Renderer {

}

pub struct Controls {

}