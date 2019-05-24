use crate::dom_factory::{
    add_event, add_style, body, button, cancel_animation_frame, document, min_max_input,
    request_animation_frame, set_timeout, window,
};
use crate::{Pattern, Universe};
use js_sys::Math;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, Element, HtmlCanvasElement, HtmlElement, HtmlInputElement, MouseEvent,
};

const CELL_SIZE: u32 = 5; // px
const GRID_COLOR: &'static str = "#222";
const DEAD_COLOR: &'static str = "#333";
const ALIVE_COLOR: &'static str = "#DDD";

pub fn get_root_style() -> String {
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
}}",
        GRID_COLOR, DEAD_COLOR, ALIVE_COLOR
    )
}

const layout: &'static str = "
    <button id='pause'></button>
    <input type='number'></input>
    <button id='write'></button>
";

const STYLE: &'static str = "
body {
    background: var(--bg);
    font: 16px/1.5 sans-serif;
}
button {
    background: var(--bg-light);
    border-radius: var(--radius);
    border: 1px solid var(--bg-lighter);
    padding: var(--pad);
    line-height: 1;
    margin: var(--pad);
}
button i {
    color: var(--fg);
    line-height: 1;
    display: block;
}
button::-moz-focus-inner {
    border: 0;
}
section.draw canvas {
    cursor: crosshair;
}
section.erase canvas {
    cursor: cell;
}
canvas {
    display: block;
}
input[type=number] {
    background: var(--fg);
    color: var(--bg);
    width: 2.5em;
    padding: var(--pad);
    margin: 0;
    border: 0;
    border-radius: var(--radius) 0 0 var(--radius);
}
input[type=number]:last-of-type::before {
    content: \"×\";
    color: red;
}
input[type=number]:last-of-type{
    border-radius: 0;
}
input::-webkit-outer-spin-button,
input::-webkit-inner-spin-button {
    -webkit-appearance: none;
}
input[type=number] {
    -moz-appearance: textfield;
}
";

pub fn get_rangle_stype() -> String {
    let style: &'static str = "
        input[type=range] {
            -webkit-appearance: none;
            width: 10em;
            background: transparent;
        }
        input[type=range]:focus {
            outline: none;
        }
        input[type=range]::-webkit-slider-thumb {
            -webkit-appearance: none;
            margin-top: -8px;
        }
        input[type=range]::-moz-range-thumb {
            border: 0;
        }
    ";
    let track: &'static str = " {
        width: 100%;
        height: 4px;
        cursor: pointer;
        animate: 1.0s;
        background: var(--bg-light);
        border-radius: 20px;
        box-sizing: content-box;
        border: 1px solid var(--bg-lighter);
    }";
    let thumb: &'static str = "{
        background: var(--fg);
        height: 20px;
        width: 20px;
        border-radius: 50%;
        cursor: pointer;
    }";
    format!(
        "{}{}{}",
        format!("
            input[type=range]::-moz-range-track{}
            input[type=range]::-webkit-slider-runnable-track{}",
        track, track),
        format!("
            input[type=range]::-moz-range-thumb{} 
            input[type=range]::-webkit-slider-thumb{}",
        thumb, thumb),
        style
    )
}

#[derive(PartialEq)]
pub enum DrawState {
    DRAW,
    ERASE,
    PLACE(Pattern),
    NONE,
}

#[derive(PartialEq)]
pub enum GameEvents {
    PAUSE,
    DRAW(DrawState),
    RANDOM,
    DELAY,
    CLEAR,
    DUMMY,
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
    root: Element,
    ui_elements: Vec<(GameEvents, HtmlElement)>,
}

impl Game {
    pub fn new() -> Self {
        let width = 64;
        let height = 64;
        let mut universe = Universe::new(width, height);
        universe.randomize_cells();

        let document = document();
        document.set_title("Conway's Game of Life!");

        add_style(format!("{}{}{}", get_root_style().as_str(), STYLE, get_rangle_stype()).as_str());

        let c = document.create_element("canvas").unwrap();
        let canvas = c.clone().dyn_into::<HtmlCanvasElement>().unwrap();
        let canvas_rect = ((CELL_SIZE + 1) * width + 1, (CELL_SIZE + 1) * height + 1);
        canvas.set_width(canvas_rect.0);
        canvas.set_height(canvas_rect.1);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let button_pause = button("pause", "material_icon");
        let button_random = button("grain", "material_icon");
        let button_draw = button("create", "material_icon");
        let button_erase = button("border_color", "material_icon");
        let button_clear = button("delete_forever", "material_icon");
        let button_glider = button("scatter_plot", "material_icon");
        let button_gun = button("send", "material_icon");
        let delay: u32 = 0;
        let button_expand = button("fullscreen", "material_icon");
        let button_shrink = button("fullscreen_exit", "material_icon");
        let input_delay = min_max_input("range", 0, 1000, delay as i32, None);
        let input_width = min_max_input("number", 0, 1000, delay as i32, Some(100));
        let padding = span("×");
        let input_height = min_max_input("number", 0, 1000, delay as i32,Some(100));

        let mut ui_elements = Vec::new();
        ui_elements.push((GameEvents::PAUSE, button_pause));
        ui_elements.push((GameEvents::DELAY, input_delay));
        ui_elements.push((GameEvents::DRAW(DrawState::DRAW), button_draw));
        ui_elements.push((GameEvents::DRAW(DrawState::ERASE), button_erase));
        ui_elements.push((GameEvents::DUMMY, button_shrink));
        ui_elements.push((GameEvents::DUMMY, input_width));
        ui_elements.push((GameEvents::DUMMY, input_height));
        ui_elements.push((GameEvents::DUMMY, button_expand));
        ui_elements.push((GameEvents::RANDOM, button_random));
        ui_elements.push((GameEvents::CLEAR, button_clear));
        ui_elements.push((
            GameEvents::DRAW(DrawState::NONE),
            c.dyn_into::<HtmlElement>().unwrap(),
        ));
        ui_elements.push((
            GameEvents::DRAW(DrawState::PLACE(Pattern::GLIDER)),
            button_glider,
        ));
        ui_elements.push((
            GameEvents::DRAW(DrawState::PLACE(Pattern::GLIDER)),
            button_gun,
        ));

        // let ui_elements = Rc::new(RefCell::new(ui_elements));
        let root = document.create_element("section").unwrap();
        body().append_child(&root).unwrap();

        let draw_state = DrawState::NONE;

        Self {
            universe,
            paused: false,
            should_draw: false,
            delay,
            context,
            ui_elements,
            root,
            draw_state,
            canvas,
            animation_id: None,
            timeout_id: None,
        }
    }
    pub fn attach_ui_elements(&self) {
        for (_, element) in self.ui_elements.iter() {
            self.root
                .append_child(&element)
                .expect(format!("Can't append the element {:?} to body!", element).as_str());
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
    pub fn ui_elements(&self) -> &[(GameEvents, HtmlElement)] {
        &self.ui_elements[..]
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
        self.root.class_list().remove_2("draw", "erase").unwrap();
        match draw_state {
            DrawState::DRAW | DrawState::PLACE(_) => {
                self.root.class_list().add_1("draw").unwrap();
            }
            DrawState::ERASE => {
                self.root.class_list().add_1("erase").unwrap();
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
        let game = self.game.borrow();
        let els = game.ui_elements();
        for (event, element) in els.iter() {
            match event {
                GameEvents::PAUSE => {
                    let s = self.game.clone();
                    add_event(element, "click", move |_| {
                        let mut game = s.borrow_mut();
                        if game.is_paused() {
                            Self::start_animation(s.clone());
                        } else {
                            game.pause();
                        }
                    });
                }
                GameEvents::RANDOM => {
                    let s = self.game.clone();
                    add_event(element, "click", move |_| {
                        let mut game = s.borrow_mut();
                        game.randomize();
                    });
                }
                GameEvents::CLEAR => {
                    let s = self.game.clone();
                    add_event(element, "click", move |_| {
                        let mut game = s.borrow_mut();
                        game.clear();
                    });
                }
                GameEvents::DELAY => {
                    let s = self.game.clone();
                    let input = element.clone().dyn_into::<HtmlInputElement>().unwrap();
                    add_event(element, "input", move |_| {
                        let mut game = s.borrow_mut();
                        let time = input.value().parse::<u32>().unwrap();
                        game.change_speed(time);
                    });
                }
                GameEvents::DRAW(draw) => match draw {
                    DrawState::DRAW => {
                        let s = self.game.clone();
                        add_event(element, "click", move |_| {
                            let mut game = s.borrow_mut();
                            game.set_draw_state(DrawState::DRAW);
                        });
                    }
                    DrawState::ERASE => {
                        let s = self.game.clone();
                        add_event(element, "click", move |_| {
                            let mut game = s.borrow_mut();
                            game.set_draw_state(DrawState::ERASE);
                        });
                    }
                    DrawState::NONE => {
                        add_event(element, "contextmenu", move |e| {
                            e.prevent_default();
                        });

                        let s = self.game.clone();
                        add_event(element, "mousemove", move |e| {
                            let me = e.dyn_into::<MouseEvent>().unwrap();
                            let mut game = s.borrow_mut();
                            game.draw_cell(me.client_x(), me.client_y());
                        });

                        let s = self.game.clone();
                        add_event(element, "mousedown", move |e| {
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
                        add_event(element, "mouseup", move |_| {
                            let mut game = s.borrow_mut();
                            game.disable_drawing();
                        });
                    }
                    DrawState::PLACE(pattern) => match pattern {
                        Pattern::GLIDER => {
                            log!("Glider clicked");
                            let s = self.game.clone();
                            add_event(element, "click", move |_| {
                                let mut game = s.borrow_mut();
                                game.set_draw_state(DrawState::PLACE(Pattern::GLIDER));
                            });
                        }
                        Pattern::GUN => {}
                    },
                },
                GameEvents::DUMMY => (),
            }
        }
    }
}
