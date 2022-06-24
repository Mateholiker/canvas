use std::mem::swap;

use eframe::egui::Vec2 as GuiVec;
use eframe::egui::{vec2, Color32, Key, Rect, Response, Sense, Ui, Widget};

use eframe::epaint::{FontId, Rounding};

pub mod canvas_handle;
pub mod drawable;
pub mod position;
use canvas_handle::CanvasHandle;
use drawable::Response as CustomResponse;
use position::Position;

use crate::Drawable;

pub struct CanvasState {
    draw_data: Box<dyn Drawable>,
    current_cutout: Rect,
    mode: CanvasMode,
    aspect_ratio: f32,
}

impl CanvasState {
    pub fn new() -> CanvasState {
        use CanvasMode::Normal;

        let mut draw_data = Box::new(());
        let default_cutout = draw_data.get_cutout();

        CanvasState {
            draw_data,
            current_cutout: default_cutout,
            mode: Normal,
            aspect_ratio: 1.0,
        }
    }

    pub fn set_draw_data(&mut self, draw_data: Box<dyn Drawable>) {
        self.draw_data = draw_data;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn take_draw_data(&mut self) -> Box<dyn Drawable> {
        let mut draw_data: Box<dyn Drawable> = Box::new(());
        swap(&mut draw_data, &mut self.draw_data);
        draw_data
    }

    pub fn draw_data_mut(&mut self) -> &mut Box<dyn Drawable> {
        &mut self.draw_data
    }

    pub fn reset_cutout(&mut self) {
        self.current_cutout = self.draw_data.get_cutout();
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        CanvasState::new()
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

    fn manage_user_input(&mut self, ui: &Ui, gui_space: Rect, response: &Response) {
        use CanvasMode::{Dragging, Normal};
        use Key::Space;

        //draw curser position
        let painter = ui.painter();
        if let Some(curser_gui_pos) = response.hover_pos() {
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
                    self.state.reset_cutout();
                }

                //zooming
                if input.scroll_delta.y.abs() > 1.0 {
                    if let Some(curser_gui_pos) = response.hover_pos() {
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
                if response.drag_started() {
                    if let Some(hover_pos) = response.hover_pos() {
                        if gui_space.contains(hover_pos) {
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
                        gui_space,
                        self.state.current_cutout,
                        self.state.aspect_ratio,
                    );
                    let translation_raw = response.drag_delta();
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

        let response = CustomResponse::from(response);
        let canvas_handle = CanvasHandle::new(
            ui,
            self.state.current_cutout,
            gui_space,
            self.state.aspect_ratio,
        );

        //pass through
        self.state.draw_data.handle_input(&response, &canvas_handle);
    }
}

impl<'s> Widget for Canvas<'s> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(vec2(50.0, 50.0), Sense::click_and_drag());
        let gui_space = response.rect;
        ui.set_clip_rect(gui_space);
        let painter = ui.painter();

        //draw the Drawable Data
        let mut canvas_handle = CanvasHandle::new(
            ui,
            self.state.current_cutout,
            gui_space,
            self.state.aspect_ratio,
        );
        self.state.draw_data.draw(&mut canvas_handle);

        //manage user input
        self.manage_user_input(ui, gui_space, &response);

        //draw a frame around the Trajectories
        painter.rect_stroke(gui_space, 0.0, (5.0, Color32::DARK_RED));

        response
    }
}
