use eframe::egui::{vec2, Color32, Key, Rect, Response, Sense, Ui, Widget};
use eframe::egui::{Align2, Vec2 as GuiVec};
use eframe::epaint::FontId;

pub mod canvas_painter;
use canvas_painter::{CanvasPainter, Drawable, Position};

pub struct CanvasState {
    draw_data: Box<dyn Drawable>,
    current_cutout: Rect,
    mode: CanvasMode,
}

impl CanvasState {
    pub fn new(mut draw_data: impl Drawable + 'static) -> CanvasState {
        use CanvasMode::Normal;

        let default_cutout = draw_data.get_cutout();

        CanvasState {
            draw_data: Box::new(draw_data),
            current_cutout: default_cutout,
            mode: Normal,
        }
    }

    pub fn set_draw_data(&mut self, draw_data: impl Drawable + 'static) {
        self.draw_data = Box::new(draw_data);
    }

    fn reset_cutout(&mut self) {
        self.current_cutout = self.draw_data.get_cutout();
    }
}

#[derive(Debug, Clone, Copy)]
enum CanvasMode {
    Dragging,
    Normal,
}

pub struct Canvas<'s> {
    state: &'s mut CanvasState,
}

impl<'s> Canvas<'s> {
    pub fn new(state: &'s mut CanvasState) -> Canvas<'s> {
        Canvas { state }
    }

    fn manage_user_input(&mut self, ui: &Ui, canvas_space: Rect, response: &Response) {
        use CanvasMode::{Dragging, Normal};
        use Key::Space;

        //draw curser position
        let painter = ui.painter();
        if let Some(curser_overlay_pos) = response.hover_pos() {
            let position = Position::Overlay(curser_overlay_pos);
            let curser_canvas_pos =
                position.to_canvas_space(canvas_space, self.state.current_cutout);

            painter.text(
                canvas_space.min + GuiVec::from((10.0, 10.0)),
                Align2::LEFT_CENTER,
                format!("Cursor: {:?}", curser_canvas_pos),
                FontId::monospace(20.0),
                Color32::LIGHT_GRAY,
            );
        }

        let input = ui.input();
        match self.state.mode {
            Normal => {
                //reseting
                if input.key_pressed(Space) {
                    self.state.reset_cutout();
                }

                //zooming
                if input.scroll_delta.y.abs() > 1.0 {
                    if let Some(curser_overlay_pos) = response.hover_pos() {
                        //calulate the curser position in trajectory space
                        //this is the fix_point of the new cutout
                        //this means its relative position must not change
                        let position = Position::Overlay(curser_overlay_pos);
                        let fix_point = position
                            .to_canvas_space(canvas_space, self.state.current_cutout)
                            .to_vec2();

                        //one click with the mouse wheel is 50.0 in scroll_delta
                        //0.9 means that the new cutout is 90% of the old cutout
                        let zoom_factor = 0.9_f32.powf(input.scroll_delta.y / 50.0);
                        let inverse_zoom_factor = 1.0 - zoom_factor;

                        //the offset is calculated so the fix_point keeps its relative position
                        let offset = fix_point * inverse_zoom_factor
                            + zoom_factor * self.state.current_cutout.min.to_vec2();

                        let new_cutout = Rect::from_min_size(
                            offset.to_pos2(),
                            self.state.current_cutout.size() * zoom_factor,
                        );

                        self.state.current_cutout = new_cutout;
                    } //else curser not on screen so ignore the scroll
                }

                //drag detection
                if response.drag_started() {
                    if let Some(hover_pos) = response.hover_pos() {
                        if canvas_space.contains(hover_pos) {
                            //drag started
                            self.state.mode = Dragging;
                        }
                    }
                }
            }

            Dragging => {
                //change cutout
                if response.drag_released() {
                    self.state.mode = Normal;
                } else {
                    let (_padding, scaling_factor) = Position::calculate_padding_and_scaling_factor(
                        canvas_space,
                        self.state.current_cutout,
                    );
                    let new_cutout = self
                        .state
                        .current_cutout
                        .translate(-response.drag_delta() / scaling_factor);
                    self.state.current_cutout = new_cutout;
                }
            }
        }
    }
}

impl<'s> Widget for Canvas<'s> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(vec2(50.0, 50.0), Sense::click_and_drag());
        let canvas_space = response.rect;
        ui.set_clip_rect(canvas_space);
        let painter = ui.painter();

        //draw a frame around the Trajectories
        painter.rect_stroke(canvas_space, 0.0, (1.0, Color32::DARK_RED));

        //manage user input
        self.manage_user_input(ui, canvas_space, &response);

        //draw the Drawable Data
        let painter = CanvasPainter::new(ui, self.state.current_cutout, canvas_space);
        self.state.draw_data.draw(&painter);

        response
    }
}
