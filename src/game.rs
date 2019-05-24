use crate::dom_factory::{
    add_event, add_style, body, cancel_animation_frame, document, get_el_id, query,
    request_animation_frame, set_timeout, window, get_css_var,
};
use crate::{Pattern, Universe};
use js_sys::Math;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Event, HtmlCanvasElement, HtmlInputElement, MouseEvent};

use maud::html;

const CELL_SIZE: u32 = 8; // px

#[derive(PartialEq)]
pub enum DrawState {
    DRAW,
    ERASE,
    PLACE(Pattern),
    NONE,
}

pub struct Game {
    universe: Universe,
    paused: bool,
    should_draw: bool,
    draw_state: DrawState,
    delay: u32,
    context: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    animation_id: Option<i32>,
    timeout_id: Option<i32>,
}

impl Game {
    pub fn new() -> Self {
        let width = 128;
        let height = 64;
        let mut universe = Universe::new(width, height);
        universe.randomize_cells();

        let markup = html! {
            section {
                div.topbar {
                    button#play title="play/pause" {i.material-icons {"pause"}}
                    button#random title="randomize universe" {i.material-icons {"grain"}}
                    button#shrink title="shrink universe" {i.material-icons {"remove"}}
                    input#width title="specify width to resize" type="number" value="64" min="0" max="1000" step="100" {}
                    span {"×"}
                    input#height title="specify width to resize" type="number" value="64" min="0" max="1000" step="100" {}
                    button#resize title="resize universe to given dimensions" {i.material-icons {"launch"}}
                    button#expand title="expand universe" {i.material-icons {"add"}}
                    button#clear title="clear universe" {i.material-icons {"delete_forever"}}
                }
                div.center {
                    canvas{}
                }
                div.bottombar {
                    button#glider title="insert glider" {i.material-icons {"scatter_plot"}}
                    button#gun title="insert glider gun" {i.material-icons {"send"}}
                    button#pulsar title="insert pulsar" {i.material-icons {"flare"}}
                    input#delay title="change speed" type="range" value="0" min="0" max="1000" {}
                    button#draw title="draw universe" {i.material-icons {"create"}}
                    button#erase title="erase universe" {i.material-icons {"border_color"}}
                }
            }
        };

        body().set_inner_html(&markup.into_string().as_str());

        let canvas = query("canvas")
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        let canvas_rect = ((CELL_SIZE + 1) * width + 1, (CELL_SIZE + 1) * height + 1);
        canvas.set_width(canvas_rect.0);
        canvas.set_height(canvas_rect.1);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let draw_state = DrawState::NONE;

        Self {
            universe,
            paused: false,
            should_draw: false,
            delay: 0,
            context,
            draw_state,
            canvas,
            animation_id: None,
            timeout_id: None,
        }
    }
    pub fn render(&mut self) {
        self.paused = false;
        self.universe.tick();
        self.draw_grid();
        self.draw_cells();
    }
    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(id) = self.animation_id {
            cancel_animation_frame(id)
        }
        if let Some(id) = self.timeout_id {
            window().clear_timeout_with_handle(id);
        }
    }
    pub fn enable_drawing(&mut self) {
        self.should_draw = true;
    }
    pub fn disable_drawing(&mut self) {
        self.should_draw = false;
    }
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    pub fn set_animation_id(&mut self, id: i32) {
        self.animation_id = Some(id);
    }
    pub fn set_timeout_id(&mut self, id: i32) {
        self.timeout_id = Some(id);
    }
    pub fn randomize(&mut self) {
        self.universe.randomize_cells();
        self.render();
        self.pause();
    }
    pub fn clear(&mut self) {
        self.universe.clear();
        self.render();
        self.pause();
    }
    pub fn speed(&self) -> u32 {
        self.delay
    }
    pub fn change_speed(&mut self, time: u32) {
        self.delay = time;
    }
    fn calculate_target(&self, x: i32, y: i32) -> (u32, u32) {
        let bounding_rect = self.canvas.get_bounding_client_rect();
        let scale_x = self.canvas.width() as f64 / bounding_rect.width();
        let scale_y = self.canvas.width() as f64 / bounding_rect.height();

        let canvas_l = (x as f64 - bounding_rect.left()) * scale_x;
        let canvas_t = (y as f64 - bounding_rect.top()) * scale_y;

        let row = Math::min(
            Math::floor(canvas_t / (CELL_SIZE + 1) as f64),
            (self.universe.height() - 1) as f64,
        ) as u32;
        let col = Math::min(
            Math::floor(canvas_l / (CELL_SIZE + 1) as f64),
            (self.universe.width() - 1) as f64,
        ) as u32;
        (row, col)
    }
    pub fn set_draw_state(&mut self, draw_state: DrawState) {
        self.canvas.class_list().remove_2("draw", "erase").unwrap();
        match draw_state {
            DrawState::DRAW | DrawState::PLACE(_) => {
                self.canvas.class_list().add_1("draw").unwrap();
            }
            DrawState::ERASE => {
                self.canvas.class_list().add_1("erase").unwrap();
            }
            _ => (),
        }
        self.draw_state = draw_state;
    }
    pub fn draw_cell(&mut self, x: i32, y: i32) {
        if !self.should_draw || self.draw_state == DrawState::NONE {
            return;
        }
        let (row, col) = self.calculate_target(x, y);
        match &self.draw_state {
            DrawState::DRAW => {
                self.universe.birth_cell(row, col);
            }
            DrawState::ERASE => {
                self.universe.kill_cell(row, col);
            }
            DrawState::PLACE(pattern) => {
                self.universe.insert(*pattern, col, row);
            }
            _ => (),
        }
        self.draw_cells();
    }
    fn draw_cells(&mut self) {
        self.context.begin_path();

        for row in 0..self.universe.height() {
            for col in 0..self.universe.width() {
                let alive = self.universe.is_alive(row, col);
                let alive_color = JsValue::from_str(get_css_var("--fg").as_str());
                let dead_color = JsValue::from_str(get_css_var("--bg-light").as_str());
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
            .set_stroke_style(&JsValue::from_str(get_css_var("--bg").as_str()));

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
    fn start_animation(game: Rc<RefCell<Game>>) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let s = game.clone();

        *f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let another_s = s.clone();
            let mut game = s.borrow_mut();
            let g = g.clone();
            let timeout_id = set_timeout(
                move || {
                    let mut game = another_s.borrow_mut();
                    game.render();
                    game.set_animation_id(request_animation_frame(g.borrow().as_ref().unwrap()));
                },
                game.speed(),
            );
            game.set_timeout_id(timeout_id);
        }) as Box<FnMut()>));
        request_animation_frame(f.borrow().as_ref().unwrap());
    }
    pub fn render_loop(&self) {
        Self::start_animation(self.game.clone());
    }
    pub fn attach_events(&self) {
        let s = self.game.clone();
        add_event(&get_el_id("play"), "click", move |_| {
            let mut game = s.borrow_mut();
            if game.is_paused() {
                Self::start_animation(s.clone());
            } else {
                game.pause();
            }
        });
        let s = self.game.clone();
        add_event(&get_el_id("delay"), "input", move |e| {
            let mut game = s.borrow_mut();
            let time = e
                .target()
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap()
                .value()
                .parse::<u32>()
                .unwrap();
            game.change_speed(time);
        });
        let s = self.game.clone();
        add_event(&get_el_id("random"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.randomize();
        });
        let s = self.game.clone();
        add_event(&get_el_id("clear"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.clear();
        });
        let s = self.game.clone();
        add_event(&get_el_id("draw"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.set_draw_state(DrawState::DRAW);
        });
        let s = self.game.clone();
        add_event(&get_el_id("erase"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.set_draw_state(DrawState::ERASE);
        });
        let s = self.game.clone();
        add_event(&get_el_id("erase"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.set_draw_state(DrawState::ERASE);
        });
        let canvas = query("canvas");
        add_event(&canvas, "contextmenu", move |e| {
            e.prevent_default();
        });
        let s = self.game.clone();
        add_event(&canvas, "mousemove", move |e| {
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let mut game = s.borrow_mut();
            game.draw_cell(me.client_x(), me.client_y());
        });
        let s = self.game.clone();
        add_event(&canvas, "mousedown", move |e| {
            let mut game = s.borrow_mut();
            let me = e.dyn_into::<MouseEvent>().unwrap();
            let btn = me.button();
            if btn == 0 {
                game.enable_drawing();
                game.draw_cell(me.client_x(), me.client_y());
            } else if btn == 2 {
                game.set_draw_state(DrawState::NONE);
            }
        });
        let s = self.game.clone();
        add_event(&canvas, "mouseup", move |_| {
            let mut game = s.borrow_mut();
            game.disable_drawing();
        });
        let s = self.game.clone();
        add_event(&get_el_id("glider"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.set_draw_state(DrawState::PLACE(Pattern::GLIDER));
        });
        let s = self.game.clone();
        add_event(&get_el_id("gun"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.set_draw_state(DrawState::PLACE(Pattern::GUN));
        });
        let s = self.game.clone();
        add_event(&get_el_id("pulsar"), "click", move |_| {
            let mut game = s.borrow_mut();
            game.set_draw_state(DrawState::PLACE(Pattern::PULSAR));
        });
    }
}
