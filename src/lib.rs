use eframe::egui::Vec2 as GuiVec;
use eframe::egui::{vec2, Color32, Key, Rect, Response as EguiResponse, Sense, Ui, Widget};

use eframe::epaint::{FontId, Rounding};

mod canvas_handle;
mod drawable;
mod position;

mod utility {
    pub mod coordinate_system;
}

pub use utility::coordinate_system::{Alignment, Axis, CoordinateSystem, Placement, Tick};

pub use canvas_handle::CanvasHandle;
pub use drawable::{Drawable, Response};
pub use position::Position;

pub struct CanvasState {
    current_cutout: Rect,
    mode: CanvasMode,
    draw_frame: bool,
    aspect_ratio: f32,
}

impl CanvasState {
    pub fn new() -> CanvasState {
        use CanvasMode::Normal;

        let default_cutout = ().get_cutout(&());

        CanvasState {
            current_cutout: default_cutout,
            mode: Normal,
            draw_frame: false,
            aspect_ratio: 1.0,
        }
    }

    pub fn draw_frame(mut self, enabled: bool) -> Self {
        self.draw_frame = enabled;
        self
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    fn reset_cutout<D, E>(&mut self, drawable: &mut E, draw_data: &D)
    where
        E: Drawable<DrawData = D>,
    {
        self.current_cutout = drawable.get_cutout(draw_data);
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        CanvasState::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanvasMode {
    Dragging,
    Normal,
}

pub struct Canvas<'s, D, E: Drawable<DrawData = D>> {
    state: &'s mut CanvasState,
    drawable: &'s mut E,
    draw_data: &'s D,
}

impl<'s, D, E: Drawable<DrawData = D>> Canvas<'s, D, E> {
    pub fn new(
        state: &'s mut CanvasState,
        drawable: &'s mut E,
        draw_data: &'s D,
    ) -> Canvas<'s, D, E> {
        Canvas {
            state,
            drawable,
            draw_data,
        }
    }

    pub fn reset_cutout(&mut self) {
        self.state.reset_cutout(self.drawable, self.draw_data)
    }

    fn manage_user_input(
        &mut self,
        ui: &mut Ui,
        gui_space: Rect,
        egui_response: &mut EguiResponse,
    ) {
        use CanvasMode::{Dragging, Normal};
        use Key::Space;

        //draw curser position
        let painter = ui.painter();
        if let Some(curser_gui_pos) = egui_response.hover_pos() {
            let position = Position::Gui(curser_gui_pos);
            let curser_canvas_pos = position.to_canvas_space(
                gui_space,
                self.state.current_cutout,
                self.state.aspect_ratio,
            );

            let galley = painter.layout_no_wrap(
                format!("Cursor: {:?}", curser_canvas_pos),
                FontId::monospace(20.0),
                Color32::LIGHT_GRAY,
            );

            let pos = gui_space.min + GuiVec::from((10.0, 10.0));

            let size = galley.size() + GuiVec::from((10.0, 10.0));
            painter.rect_filled(
                Rect::from_min_size(pos, size),
                Rounding::same(2.0),
                Color32::DARK_BLUE,
            );
            painter.galley(pos + GuiVec::from((5.0, 5.0)), galley);
        }

        let input = ui.input();
        match self.state.mode {
            Normal => {
                //reseting
                if input.key_pressed(Space) {
                    self.reset_cutout();
                }

                //zooming
                if input.scroll_delta.y.abs() > 1.0 {
                    if let Some(curser_gui_pos) = egui_response.hover_pos() {
                        //calulate the curser position in trajectory space
                        //this is the fix_point of the new cutout
                        //this means its relative position must not change
                        let position = Position::Gui(curser_gui_pos);
                        let fix_point = position
                            .to_canvas_space(
                                gui_space,
                                self.state.current_cutout,
                                self.state.aspect_ratio,
                            )
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
                if egui_response.drag_started() {
                    if let Some(hover_pos) = egui_response.hover_pos() {
                        if gui_space.contains(hover_pos) {
                            //drag started
                            self.state.mode = Dragging;
                        }
                    }
                }
            }

            Dragging => {
                //change cutout
                if egui_response.drag_released() {
                    self.state.mode = Normal;
                } else {
                    let (_padding, scaling_factor) = Position::calculate_padding_and_scaling_factor(
                        gui_space,
                        self.state.current_cutout,
                        self.state.aspect_ratio,
                    );
                    let translation_raw = egui_response.drag_delta();
                    let translation_scaled = GuiVec {
                        x: translation_raw.x / scaling_factor.x(),
                        y: translation_raw.y / scaling_factor.y(),
                    };
                    let translation_rotated = GuiVec {
                        x: -translation_scaled.x,
                        y: translation_scaled.y,
                    };
                    let new_cutout = self.state.current_cutout.translate(translation_rotated);
                    self.state.current_cutout = new_cutout;
                }
            }
        }
        drop(input);

        let response = Response::from(&*egui_response);
        let canvas_handle = CanvasHandle::new(
            ui,
            egui_response,
            self.state.current_cutout,
            gui_space,
            self.state.aspect_ratio,
        );

        //pass through
        self.drawable.handle_input(&response, &canvas_handle);
    }
}

impl<'s, D, E: Drawable<DrawData = D>> Widget for Canvas<'s, D, E> {
    fn ui(mut self, ui: &mut Ui) -> EguiResponse {
        let mut response = ui.allocate_response(vec2(50.0, 50.0), Sense::click_and_drag());
        let gui_space = response.rect;
        ui.set_clip_rect(gui_space);

        //draw the Drawable Data
        let mut canvas_handle = CanvasHandle::new(
            ui,
            &mut response,
            self.state.current_cutout,
            gui_space,
            self.state.aspect_ratio,
        );
        self.drawable.draw(&mut canvas_handle, self.draw_data);

        //manage user input
        self.manage_user_input(ui, gui_space, &mut response);

        if self.state.draw_frame {
            //draw a frame around the Trajectories
            let painter = ui.painter();
            painter.rect_stroke(gui_space, 0.0, (5.0, Color32::DARK_RED));
        }

        response
    }
}
