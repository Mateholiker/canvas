use eframe::egui::{Color32, Rect, Stroke, Ui};
use eframe::emath::Align2;
use eframe::epaint::FontId;
use simple_math::{Rectangle, Vec2};

use crate::Position;

///mirrors the guidd
pub struct CanvasHandle<'p> {
    ui: &'p Ui,
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

    pub fn bounding_box(&self) -> Rectangle {
        let gui_rect = self.ui.painter().clip_rect();
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
        self.ui.painter().line_segment(points, stroke);
    }

    pub fn circle_filled(&mut self, center: Position, radius: f32, fill_color: impl Into<Color32>) {
        let center = center.to_gui_space(self.gui_space, self.current_cutout, self.aspect_ratio);
        self.ui.painter().circle_filled(center, radius, fill_color);
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
}
