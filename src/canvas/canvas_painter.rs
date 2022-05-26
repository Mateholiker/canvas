use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use eframe::egui::Vec2 as GuiVec;
use eframe::egui::{Color32, Context, Painter, Pos2, Rect, Response, Stroke, Ui};
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

    fn to_gui_space(self, gui_space: Rect, current_cutout: Rect) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        match self {
            Canvas(_) => {
                let overlay = Overlay(self.to_overlay_space(gui_space, current_cutout));
                overlay.to_gui_space(gui_space, current_cutout)
            }

            Overlay(pos) => Pos2 {
                x: pos.x,
                y: gui_space.max.y - pos.y + gui_space.min.y,
            },

            Gui(pos) => pos,
        }
    }

    fn to_overlay_space(self, gui_space: Rect, current_cutout: Rect) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        let (padding, scaling_factor) =
            Position::calculate_padding_and_scaling_factor(gui_space, current_cutout);
        match self {
            Canvas(pos) => {
                let new_vec = (pos.to_vec2() - current_cutout.min.to_vec2()) * scaling_factor
                    + padding
                    + gui_space.min.to_vec2();
                new_vec.to_pos2()
            }
            Overlay(pos) => pos,

            Gui(pos) => Pos2 {
                x: pos.x,
                y: gui_space.max.y - pos.y + gui_space.min.y,
            },
        }
    }

    pub(super) fn to_canvas_space(self, gui_space: Rect, current_cutout: Rect) -> Pos2 {
        use Position::{Canvas, Gui, Overlay};
        let (padding, scaling_factor) =
            Position::calculate_padding_and_scaling_factor(gui_space, current_cutout);
        match self {
            Canvas(pos) => pos,

            Overlay(pos) => {
                let canvas_vec = (pos.to_vec2() - padding - gui_space.min.to_vec2())
                    / scaling_factor
                    + current_cutout.min.to_vec2();
                canvas_vec.to_pos2()
            }

            Gui(_) => {
                let overlay = Overlay(self.to_overlay_space(gui_space, current_cutout));
                overlay.to_canvas_space(gui_space, current_cutout)
            }
        }
    }

    pub(super) fn calculate_padding_and_scaling_factor(
        gui_space: Rect,
        current_cutout: Rect,
    ) -> (GuiVec, f32) {
        //calulate the rations of the spaces
        let ratio_trajectories = current_cutout.aspect_ratio();
        let ratio_canvas = gui_space.shrink(MIN_PADDING).aspect_ratio();

        //calulate the scaling factor and padding
        let scaling_factor;
        let x_padding;
        let y_padding;
        if ratio_trajectories < ratio_canvas {
            // y-Axe is limiting
            scaling_factor = gui_space.shrink(MIN_PADDING).height() / current_cutout.height();
            x_padding = (gui_space.width() - current_cutout.width() * scaling_factor) / 2.0;
            y_padding = MIN_PADDING;
        } else {
            // x-Axe is limiting
            scaling_factor = gui_space.shrink(MIN_PADDING).width() / current_cutout.width();
            x_padding = MIN_PADDING;
            y_padding = (gui_space.height() - current_cutout.height() * scaling_factor) / 2.0;
        }

        //get padding vector
        let padding = GuiVec {
            x: x_padding,
            y: y_padding,
        };

        (padding, scaling_factor)
    }
}

///mirrors the guidd
pub struct CanvasPainter<'p> {
    painter: &'p Painter,
    current_cutout: Rect,
    gui_space: Rect,
}

impl<'p> CanvasPainter<'p> {
    pub(super) fn new(ui: &Ui, current_cutout: Rect, gui_space: Rect) -> CanvasPainter {
        CanvasPainter {
            painter: ui.painter(),
            current_cutout,
            gui_space,
        }
    }

    pub fn convert_to_overlay_space(&self, pos: Position) -> Position {
        Position::Overlay(pos.to_overlay_space(self.gui_space, self.current_cutout))
    }

    pub fn convert_to_canvas_space(&self, pos: Position) -> Position {
        Position::Canvas(pos.to_canvas_space(self.gui_space, self.current_cutout))
    }

    pub fn bounding_box(&self) -> Rectangle {
        let gui_rect = self.painter.clip_rect();
        Rectangle::new(gui_rect.max.into(), gui_rect.min.into())
    }

    pub fn line_segment(&self, points: (Position, Position), stroke: impl Into<Stroke>) {
        let points = [
            points.0.to_gui_space(self.gui_space, self.current_cutout),
            points.1.to_gui_space(self.gui_space, self.current_cutout),
        ];
        self.painter.line_segment(points, stroke);
    }

    pub fn circle_filled(&self, center: Position, radius: f32, fill_color: impl Into<Color32>) {
        let center = center.to_gui_space(self.gui_space, self.current_cutout);
        self.painter.circle_filled(center, radius, fill_color);
    }

    pub fn text(
        &self,
        pos: Position,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
    ) {
        let pos = pos.to_gui_space(self.gui_space, self.current_cutout);
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

pub trait Drawable {
    fn draw(&mut self, painter: &CanvasPainter);

    fn get_cutout(&mut self) -> Rect;

    #[allow(unused_variables)]
    fn handle_input(&mut self, ctx: &Context, response: &Response) {}
}

impl<T> Drawable for Vec<T>
where
    T: Drawable,
{
    fn draw(&mut self, painter: &CanvasPainter) {
        for drawable in self {
            drawable.draw(painter);
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
    fn handle_input(&mut self, ctx: &Context, response: &Response) {
        for drawable in self {
            drawable.handle_input(ctx, response);
        }
    }
}

impl Drawable for () {
    fn draw(&mut self, _painter: &CanvasPainter) {}

    fn get_cutout(&mut self) -> Rect {
        //dummy value
        Rect::from_two_pos((0.0, 0.0).into(), (10.0, 10.0).into())
    }
}

impl<T> Drawable for Rc<RefCell<T>>
where
    T: Drawable,
{
    fn draw(&mut self, painter: &CanvasPainter) {
        let mut borrow = self.borrow_mut();
        borrow.draw(painter);
    }

    fn get_cutout(&mut self) -> Rect {
        let mut borrow = self.borrow_mut();
        borrow.get_cutout()
    }

    fn handle_input(&mut self, ctx: &Context, response: &Response) {
        let mut borrow = self.borrow_mut();
        borrow.handle_input(ctx, response);
    }
}

impl<T> Drawable for Box<T>
where
    T: Drawable,
{
    fn draw(&mut self, painter: &CanvasPainter) {
        self.deref_mut().draw(painter);
    }

    fn get_cutout(&mut self) -> Rect {
        self.deref_mut().get_cutout()
    }

    fn handle_input(&mut self, ctx: &Context, response: &Response) {
        self.deref_mut().handle_input(ctx, response);
    }
}

impl<T, G> Drawable for (T, G)
where
    T: Drawable,
    G: Drawable,
{
    fn draw(&mut self, painter: &CanvasPainter) {
        self.0.draw(painter);
        self.1.draw(painter);
    }

    fn get_cutout(&mut self) -> Rect {
        let rect0 = self.0.get_cutout();
        let rect1 = self.1.get_cutout();

        rect0.union(rect1)
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, ctx: &Context, response: &Response) {
        self.0.handle_input(ctx, response);
        self.1.handle_input(ctx, response);
    }
}
