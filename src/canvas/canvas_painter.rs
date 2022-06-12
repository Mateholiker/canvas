use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use eframe::egui::Vec2 as GuiVec;
use eframe::egui::{Color32, Painter, Pos2, Rect, Response as EGuiResponse, Stroke, Ui};
use eframe::emath::Align2;
use eframe::epaint::FontId;
use simple_math::{Rectangle, Vec2};

const MIN_PADDING: f32 = 20.0;

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Gui(Pos2),
    Overlay(Pos2),
    Canvas(Pos2),
}

impl Position {
    pub fn get_raw_pos(self) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        let (Gui(pos) | Overlay(pos) | Canvas(pos)) = self;
        pos
    }

    fn to_gui_space(self, gui_space: Rect, current_cutout: Rect, aspect_ratio: f32) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        match self {
            Canvas(_) => {
                let overlay =
                    Overlay(self.to_overlay_space(gui_space, current_cutout, aspect_ratio));
                overlay.to_gui_space(gui_space, current_cutout, aspect_ratio)
            }

            Overlay(pos) => Pos2 {
                x: pos.x,
                y: gui_space.max.y - pos.y + gui_space.min.y,
            },

            Gui(pos) => pos,
        }
    }

    fn to_overlay_space(self, gui_space: Rect, current_cutout: Rect, aspect_ratio: f32) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        let (padding, scaling_factor) =
            Position::calculate_padding_and_scaling_factor(gui_space, current_cutout, aspect_ratio);
        match self {
            Canvas(pos) => {
                let padding: GuiVec = padding.into();
                let canvas_vec_moved = pos.to_vec2() - current_cutout.min.to_vec2();
                let canvas_vec_scaled = GuiVec {
                    x: canvas_vec_moved.x * scaling_factor.x(),
                    y: canvas_vec_moved.y * scaling_factor.y(),
                };
                let overlay_vec = canvas_vec_scaled + padding + gui_space.min.to_vec2();
                overlay_vec.to_pos2()
            }
            Overlay(pos) => pos,

            Gui(pos) => Pos2 {
                x: pos.x,
                y: gui_space.max.y - pos.y + gui_space.min.y,
            },
        }
    }

    pub(super) fn to_canvas_space(
        self,
        gui_space: Rect,
        current_cutout: Rect,
        aspect_ratio: f32,
    ) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        let (padding, scaling_factor) =
            Position::calculate_padding_and_scaling_factor(gui_space, current_cutout, aspect_ratio);
        match self {
            Canvas(pos) => pos,

            Overlay(pos) => {
                let padding: GuiVec = padding.into();
                let overlay_vec_moved = pos.to_vec2() - padding - gui_space.min.to_vec2();
                let overlay_vec_scaled = GuiVec {
                    x: overlay_vec_moved.x / scaling_factor.x(),
                    y: overlay_vec_moved.y / scaling_factor.y(),
                };
                let canvas_vec = overlay_vec_scaled + current_cutout.min.to_vec2();
                canvas_vec.to_pos2()
            }

            Gui(_) => {
                let overlay =
                    Overlay(self.to_overlay_space(gui_space, current_cutout, aspect_ratio));
                overlay.to_canvas_space(gui_space, current_cutout, aspect_ratio)
            }
        }
    }

    pub(super) fn calculate_padding_and_scaling_factor(
        gui_space: Rect,
        current_cutout: Rect,
        aspect_ratio: f32,
    ) -> (Vec2, Vec2) {
        //calulate the rations of the spaces
        let ratio_trajectories = current_cutout.aspect_ratio() * aspect_ratio;
        let ratio_canvas = gui_space.shrink(MIN_PADDING).aspect_ratio();

        let (x_stretch, y_stretch) = if aspect_ratio > 1.0 {
            (aspect_ratio, 1.0)
        } else {
            (1.0, 1.0 / aspect_ratio)
        };

        //calulate the scaling factor and padding
        let scaling_factor;
        let x_padding;
        let y_padding;
        if ratio_trajectories < ratio_canvas {
            // y-Axe is limiting
            scaling_factor =
                gui_space.shrink(MIN_PADDING).height() / (current_cutout.height() * y_stretch);
            x_padding = (gui_space.width() - current_cutout.width() * scaling_factor) / 2.0;
            y_padding = MIN_PADDING;
        } else {
            // x-Axe is limiting
            scaling_factor =
                gui_space.shrink(MIN_PADDING).width() / (current_cutout.width() * x_stretch);
            x_padding = MIN_PADDING;
            y_padding = (gui_space.height() - current_cutout.height() * scaling_factor) / 2.0;
        }
        let x_scaling_factor = scaling_factor * x_stretch;
        let y_scaling_factor = scaling_factor * y_stretch;

        //get padding vector
        let padding = Vec2::new(x_padding, y_padding);
        let scaling_factor = Vec2::new(x_scaling_factor, y_scaling_factor);

        (padding, scaling_factor)
    }
}

///mirrors the guidd
pub struct CanvasHandle<'p> {
    painter: &'p Painter,
    current_cutout: Rect,
    gui_space: Rect,
    aspect_ratio: f32,
}

impl<'p> CanvasHandle<'p> {
    pub(super) fn new(
        ui: &Ui,
        current_cutout: Rect,
        gui_space: Rect,
        aspect_ratio: f32,
    ) -> CanvasHandle {
        CanvasHandle {
            painter: ui.painter(),
            current_cutout,
            gui_space,
            aspect_ratio,
        }
    }

    pub fn convert_to_overlay_space(&self, pos: Position) -> Position {
        Position::Overlay(pos.to_overlay_space(
            self.gui_space,
            self.current_cutout,
            self.aspect_ratio,
        ))
    }

    pub fn convert_to_canvas_space(&self, pos: Position) -> Position {
        Position::Canvas(pos.to_canvas_space(
            self.gui_space,
            self.current_cutout,
            self.aspect_ratio,
        ))
    }

    pub fn bounding_box(&self) -> Rectangle {
        let gui_rect = self.painter.clip_rect();
        Rectangle::new(gui_rect.max.into(), gui_rect.min.into())
    }

    pub fn line_segment(&mut self, points: (Position, Position), stroke: impl Into<Stroke>) {
        let points = [
            points
                .0
                .to_gui_space(self.gui_space, self.current_cutout, self.aspect_ratio),
            points
                .1
                .to_gui_space(self.gui_space, self.current_cutout, self.aspect_ratio),
        ];
        self.painter.line_segment(points, stroke);
    }

    pub fn circle_filled(&mut self, center: Position, radius: f32, fill_color: impl Into<Color32>) {
        let center = center.to_gui_space(self.gui_space, self.current_cutout, self.aspect_ratio);
        self.painter.circle_filled(center, radius, fill_color);
    }

    pub fn text(
        &mut self,
        pos: Position,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
    ) {
        let pos = pos.to_gui_space(self.gui_space, self.current_cutout, self.aspect_ratio);
        self.painter.text(pos, anchor, text, font_id, text_color);
    }

    pub fn text_size(&self, text: impl ToString, font_id: FontId) -> Vec2 {
        //color is just a dummy value
        let gally = self
            .painter
            .layout_no_wrap(text.to_string(), font_id, Color32::BLACK);
        gally.size().into()
    }
}

pub struct Response {
    pub curser_pos: Option<Position>,
    pub clicked: bool,
}

impl From<&EGuiResponse> for Response {
    fn from(response: &EGuiResponse) -> Self {
        Response {
            curser_pos: response.hover_pos().map(Position::Gui),
            clicked: response.clicked(),
        }
    }
}

pub trait Drawable {
    fn draw(&mut self, handle: &mut CanvasHandle);

    fn get_cutout(&mut self) -> Rect;

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {}
}

impl<T> Drawable for Vec<T>
where
    T: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        for drawable in self {
            drawable.draw(handle);
        }
    }

    fn get_cutout(&mut self) -> Rect {
        if let Some(first) = self.first_mut() {
            let mut rect = first.get_cutout();
            for drawable in self {
                rect = rect.union(drawable.get_cutout());
            }
            rect
        } else {
            //dummy value
            Rect::from_two_pos((0.0, 0.0).into(), (10.0, 10.0).into())
        }
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        for drawable in self {
            drawable.handle_input(response, handle);
        }
    }
}

impl Drawable for () {
    fn draw(&mut self, _handle: &mut CanvasHandle) {}

    fn get_cutout(&mut self) -> Rect {
        //dummy value
        Rect::from_two_pos((0.0, 0.0).into(), (10.0, 10.0).into())
    }
}

impl<T> Drawable for Rc<RefCell<T>>
where
    T: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        let mut borrow = self.borrow_mut();
        borrow.draw(handle);
    }

    fn get_cutout(&mut self) -> Rect {
        let mut borrow = self.borrow_mut();
        borrow.get_cutout()
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        let mut borrow = self.borrow_mut();
        borrow.handle_input(response, handle);
    }
}

impl<T> Drawable for Box<T>
where
    T: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        self.deref_mut().draw(handle);
    }

    fn get_cutout(&mut self) -> Rect {
        self.deref_mut().get_cutout()
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        self.deref_mut().handle_input(response, handle);
    }
}

impl<T, G> Drawable for (T, G)
where
    T: Drawable,
    G: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        self.0.draw(handle);
        self.1.draw(handle);
    }

    fn get_cutout(&mut self) -> Rect {
        let rect0 = self.0.get_cutout();
        let rect1 = self.1.get_cutout();

        rect0.union(rect1)
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        self.0.handle_input(response, handle);
        self.1.handle_input(response, handle);
    }
}
