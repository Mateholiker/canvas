const MIN_PADDING: f32 = 20.0;

use eframe::egui::Vec2 as GuiVec;
use eframe::egui::{Pos2, Rect};
use simple_math::Vec2;

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

    pub(crate) fn to_gui_space(
        self,
        gui_space: Rect,
        current_cutout: Rect,
        aspect_ratio: f32,
    ) -> Pos2 {
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

    pub(crate) fn to_overlay_space(
        self,
        gui_space: Rect,
        current_cutout: Rect,
        aspect_ratio: f32,
    ) -> Pos2 {
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

    pub(crate) fn to_canvas_space(
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
            x_padding =
                (gui_space.width() - current_cutout.width() * scaling_factor * x_stretch) / 2.0;
            y_padding = MIN_PADDING;
        } else {
            // x-Axe is limiting
            scaling_factor =
                gui_space.shrink(MIN_PADDING).width() / (current_cutout.width() * x_stretch);
            x_padding = MIN_PADDING;
            y_padding =
                (gui_space.height() - current_cutout.height() * scaling_factor * y_stretch) / 2.0;
        }
        let x_scaling_factor = scaling_factor * x_stretch;
        let y_scaling_factor = scaling_factor * y_stretch;

        //get padding vector
        let padding = Vec2::new(x_padding, y_padding);
        let scaling_factor = Vec2::new(x_scaling_factor, y_scaling_factor);

        (padding, scaling_factor)
    }
}
