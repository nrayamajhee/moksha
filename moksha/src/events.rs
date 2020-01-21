#[derive(Debug, Copy, Clone)]
pub struct Events {
    pub viewport: ViewportEvent,
    pub canvas: CanvasEvent,
}

impl Events {
    pub fn new() -> Self {
        Events{
            viewport: ViewportEvent::None,
            canvas: CanvasEvent::Point,
        }
    }
    pub fn clear(&mut self) {
        self.viewport = ViewportEvent::None;
        if self.canvas == CanvasEvent::Resize {
            self.canvas = CanvasEvent::Point;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ViewportEvent {
    Rotate(i32,i32),
    Zoom(f64),
    None,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CanvasEvent{
    Zoom,
    Point,
    Grab,
    Resize,
}


