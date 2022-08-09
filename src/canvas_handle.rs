use eframe::egui::{Color32, Image, Rect, Stroke, Ui};
use eframe::emath::{Align2, Pos2};
use eframe::epaint::{FontId, Rounding};
use egui_extras::RetainedImage;
use simple_math::{Rectangle, Vec2};

use crate::Position;

///mirrors the gui
pub struct CanvasHandle<'p> {
    ui: &'p mut Ui,
    current_cutout: Rect,
    gui_space: Rect,
    aspect_ratio: f32,
}

impl<'p> CanvasHandle<'p> {
    pub(super) fn new(
        ui: &mut Ui,
        current_cutout: Rect,
        gui_space: Rect,
        aspect_ratio: f32,
    ) -> CanvasHandle {
        CanvasHandle {
            ui,
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

    fn convert_to_gui_space(&self, pos: Position) -> Pos2 {
        pos.to_gui_space(self.gui_space, self.current_cutout, self.aspect_ratio)
    }

    pub fn bounding_box(&self) -> Rectangle {
        let gui_rect = self.ui.painter().clip_rect();
        Rectangle::new(gui_rect.max.into(), gui_rect.min.into())
    }

    pub fn line_segment(&mut self, points: (Position, Position), stroke: impl Into<Stroke>) {
        let points = [
            self.convert_to_gui_space(points.0),
            self.convert_to_gui_space(points.1),
        ];
        self.ui.painter().line_segment(points, stroke);
    }

    pub fn circle_filled(&mut self, center: Position, radius: f32, fill_color: impl Into<Color32>) {
        let center = self.convert_to_gui_space(center);
        self.ui.painter().circle_filled(center, radius, fill_color);
    }

    pub fn rect(
        &mut self,
        corner_a: Position,
        corner_b: Position,
        rounding: impl Into<Rounding>,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) {
        let corner_a = self.convert_to_gui_space(corner_a);
        let corner_b = self.convert_to_gui_space(corner_b);
        let rect = Rect::from_two_pos(corner_a, corner_b);

        self.ui.painter().rect(rect, rounding, fill_color, stroke);
    }

    pub fn text(
        &mut self,
        pos: Position,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        text_color: Color32,
    ) {
        let pos = self.convert_to_gui_space(pos);
        self.ui
            .painter()
            .text(pos, anchor, text, font_id, text_color);
    }

    pub fn text_size(&self, text: impl ToString, font_id: FontId) -> Vec2 {
        //color is just a dummy value
        let gally = self
            .ui
            .painter()
            .layout_no_wrap(text.to_string(), font_id, Color32::BLACK);
        gally.size().into()
    }

    pub fn request_repaint(&self) {
        self.ui.ctx().request_repaint();
    }

    ///returns the time in seconds relatvie to something
    pub fn time(&self) -> f64 {
        self.ui.ctx().input().time
    }

    pub fn cursor_pos(&self) -> Option<Position> {
        self.ui.ctx().input().pointer.hover_pos().map(Position::Gui)
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio
    }

    pub fn image(&mut self, image: &RetainedImage, corner_a: Position, corner_b: Position) {
        let a = self.convert_to_gui_space(corner_a);
        let b = self.convert_to_gui_space(corner_b);

        let [x, y] = image.size();
        let image = Image::new(image.texture_id(self.ui.ctx()), (x as f32, y as f32));

        image.paint_at(self.ui, Rect::from_two_pos(a, b));
    }

    /// returs the Rectangle in the canvas space that is currently visual
    /// in general, this is not equal to the current cutout
    /// but bigger in one dimension
    pub fn get_draw_region_in_canvas_space(&self) -> Rectangle {
        let corner_a = Position::Gui(self.gui_space.min);
        let corner_b = Position::Gui(self.gui_space.max);

        let corner_a = self.convert_to_canvas_space(corner_a).get_raw_pos().into();
        let corner_b = self.convert_to_canvas_space(corner_b).get_raw_pos().into();

        Rectangle::new(corner_a, corner_b)
    }

    pub fn dark_mode(&self) -> bool {
        self.ui.style().visuals.dark_mode
    }
}
