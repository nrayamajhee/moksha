use crate::dom_factory::{add_events, button, request_animation_frame, window};
use crate::Universe;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, Document, EventTarget, HtmlBodyElement, HtmlCanvasElement,
    HtmlElement, HtmlStyleElement, Node,
};

const CELL_SIZE: u32 = 5; // px
const GRID_COLOR: &'static str = "#222";
const DEAD_COLOR: &'static str = "#333";
const ALIVE_COLOR: &'static str = "#DDD";

pub fn get_style() -> String {
    format!(
        "
@import url('https://fonts.googleapis.com/icon?family=Material+Icons');
:root {{
    --bg: {};
    --bg-light: {};
    --fg: {};
    --bg-lighter: #444;
    --radius: 10px;
    --pad: 10px;
    --gap: 20px;
}}
body {{
    background: var(--bg);
    display: flex;
    height: 100vh;
    justify-content: center;
    align-items: center;
}}
button {{
    background: var(--bg-light);
    border-radius: var(--radius);
    border: 1px solid var(--bg-lighter);
    padding: var(--pad);
    line-height: 1;
    margin: var(--pad);
}}
button i {{
    color: var(--fg);
    line-height: 1;
    display: block;
}}
", GRID_COLOR, DEAD_COLOR, ALIVE_COLOR)
}

#[derive(Hash, Eq, PartialEq)]
pub enum GameEvents {
    PAUSE,
    RANDOM,
}

pub struct Game {
    universe: Universe,
    paused: bool,
    context: CanvasRenderingContext2d,
    ui_elements: Rc<RefCell<HashMap<GameEvents, HtmlElement>>>,
}

impl Game {
    pub fn new() -> Self {
        let width = 64;
        let height = 64;
        let universe = Universe::new(width, height);

        let document = web_sys::window().unwrap().document().unwrap();
        document.set_title("Conway's Game of Life!");

        let style_sheet = document
            .create_element("style")
            .unwrap()
            .dyn_into::<HtmlStyleElement>()
            .unwrap();
        let body = document.body().unwrap();
        let canvas = document.create_element("canvas").unwrap();

        let button_pause = button("pause_circle_filled", "material_icon");
        let button_random = button("gradient", "material_icon");

        style_sheet.set_type("text/css");
        style_sheet.set_inner_html(get_style().as_str());

        let body = body.dyn_into::<HtmlBodyElement>().unwrap();
        let head = document.dyn_into::<Document>().unwrap().head().unwrap();

        head.append_child(&style_sheet.dyn_into::<Node>().unwrap())
            .unwrap();
        body.append_child(&canvas).unwrap();
        body.append_child(&button_pause).unwrap();
        body.append_child(&button_random).unwrap();

        let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        canvas.set_width((CELL_SIZE + 1) * width + 1);
        canvas.set_height((CELL_SIZE + 1) * height + 1);

        let mut ui_elements = HashMap::new();
        ui_elements.insert(GameEvents::PAUSE, button_pause);
        ui_elements.insert(GameEvents::RANDOM, button_random);

        let ui_elements = Rc::new(RefCell::new(ui_elements));

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Self {
            universe,
            paused: false,
            context,
            ui_elements,
        }
    }
    pub fn update(&mut self) {
        if !self.paused {
            self.draw_cells();
            self.draw_grid();
            self.universe.tick()
        }
    }
    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }
    pub fn toggle(&mut self) {
        if !self.paused {
            self.pause()
        } else {
            self.resume();
        };
    }
    pub fn ui_elements(&self) -> Rc<RefCell<HashMap<GameEvents, HtmlElement>>> {
        self.ui_elements.clone()
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
        self.context
            .set_stroke_style(&JsValue::from_str(GRID_COLOR));

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

pub struct Controller {
    game: Rc<RefCell<Game>>,
}

impl Controller {
    pub fn new(game: Rc<RefCell<Game>>) -> Self {
        Self { game }
    }
    pub fn update(&self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let s = self.game.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let mut game = s.borrow_mut();
            game.update();
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
    }
    pub fn attach_events(&self) {
        let els = self.game.borrow().ui_elements();
        for (event, button) in els.borrow_mut().iter_mut() {
            match event {
                GameEvents::PAUSE => {
                    let s = self.game.clone();
                    add_events(
                        button,
                        "click",
                        Box::new(move || {
                            let mut game = s.borrow_mut();
                            game.toggle();
                        }),
                    );
                }
                GameEvents::RANDOM => {
                    let s = self.game.clone();
                    add_events(
                        button,
                        "click",
                        Box::new(move || {
                            let mut game = s.borrow_mut();
                            game.pause();
                        }),
                    );
                }
            }
        }
    }
}
