use eframe::egui::Vec2 as GuiVec;
use eframe::egui::{Color32, Context, Painter, Pos2, Rect, Response, Stroke, Ui};

const MIN_PADDING: f32 = 20.0;

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Overlay(Pos2),
    Canvas(Pos2),
}

impl Position {
    pub(super) fn to_overlay_space(self, canvas_space: Rect, current_cutout: Rect) -> Pos2 {
        use Position::{Canvas, Overlay};
        let (padding, scaling_factor) =
            Position::calculate_padding_and_scaling_factor(canvas_space, current_cutout);
        match self {
            Canvas(pos) => {
                let new_vec = (pos.to_vec2() - current_cutout.min.to_vec2()) * scaling_factor
                    + padding
                    + canvas_space.min.to_vec2();
                new_vec.to_pos2()
            }
            Overlay(pos) => pos,
        }
    }

    pub(super) fn to_canvas_space(self, canvas_space: Rect, current_cutout: Rect) -> Pos2 {
        use Position::{Canvas, Overlay};
        let (padding, scaling_factor) =
            Position::calculate_padding_and_scaling_factor(canvas_space, current_cutout);
        match self {
            Canvas(pos) => pos,
            Overlay(pos) => {
                let canvas_vec = (pos.to_vec2() - padding - canvas_space.min.to_vec2())
                    / scaling_factor
                    + current_cutout.min.to_vec2();
                canvas_vec.to_pos2()
            }
        }
    }

    pub(super) fn calculate_padding_and_scaling_factor(
        canvas_space: Rect,
        current_cutout: Rect,
    ) -> (GuiVec, f32) {
        //calulate the rations of the spaces
        let ratio_trajectories = current_cutout.aspect_ratio();
        let ratio_canvas = canvas_space.shrink(MIN_PADDING).aspect_ratio();

        //calulate the scaling factor and padding
        let scaling_factor;
        let x_padding;
        let y_padding;
        if ratio_trajectories < ratio_canvas {
            // y-Axe is limiting
            scaling_factor = canvas_space.shrink(MIN_PADDING).height() / current_cutout.height();
            x_padding = (canvas_space.width() - current_cutout.width() * scaling_factor) / 2.0;
            y_padding = MIN_PADDING;
        } else {
            // x-Axe is limiting
            scaling_factor = canvas_space.shrink(MIN_PADDING).width() / current_cutout.width();
            x_padding = MIN_PADDING;
            y_padding = (canvas_space.height() - current_cutout.height() * scaling_factor) / 2.0;
        }

        //get padding vector
        let padding = GuiVec {
            x: x_padding,
            y: y_padding,
        };

        (padding, scaling_factor)
    }
}

pub struct CanvasPainter<'p> {
    painter: &'p Painter,
    current_cutout: Rect,
    canvas_space: Rect,
}

impl<'p> CanvasPainter<'p> {
    pub(super) fn new(ui: &Ui, current_cutout: Rect, canvas_space: Rect) -> CanvasPainter {
        CanvasPainter {
            painter: ui.painter(),
            current_cutout,
            canvas_space,
        }
    }

    pub fn line_segment(&self, points: (Position, Position), stroke: impl Into<Stroke>) {
        let points = [
            points
                .0
                .to_overlay_space(self.canvas_space, self.current_cutout),
            points
                .1
                .to_overlay_space(self.canvas_space, self.current_cutout),
        ];
        self.painter.line_segment(points, stroke);
    }

    pub fn circle_filled(&self, center: Position, radius: f32, fill_color: impl Into<Color32>) {
        let center = center.to_overlay_space(self.canvas_space, self.current_cutout);
        self.painter.circle_filled(center, radius, fill_color);
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
