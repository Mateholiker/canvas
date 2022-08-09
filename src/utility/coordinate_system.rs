#![allow(dead_code)]
use std::{cmp::min, marker::PhantomData};

use eframe::{
    emath::{Align2, Pos2, Rect},
    epaint::{Color32, FontFamily, FontId},
};
use simple_math::Rectangle;

use crate::{CanvasHandle, Drawable, Position};

const DEFAULT_PADDING: f32 = 60.0;
const THICK_LINE_WIDTH: f32 = 1.0;
const THIN_LINE_WIDTH: f32 = 0.5;

const MAYOR_TICK_STROKE_LENGHT: f32 = 4.0;

const MIN_NUMBER_OF_TICKS: u8 = 4;

#[derive(Debug)]
pub struct CoordinateSystem<D> {
    x_axis: Option<Axis>,
    y_axis: Option<Axis>,
    phantom: PhantomData<D>,
}

impl<D> CoordinateSystem<D> {
    pub fn new() -> CoordinateSystem<D> {
        CoordinateSystem {
            x_axis: Some(Axis::default()),
            y_axis: Some(Axis::default()),
            phantom: PhantomData,
        }
    }

    pub fn x_axis() -> CoordinateSystem<D> {
        CoordinateSystem {
            x_axis: Some(Axis::default()),
            y_axis: None,
            phantom: PhantomData,
        }
    }

    pub fn with_mayor_tick_interval(mut self, mayor_tick_interval: Tick) -> CoordinateSystem<D> {
        if let Some(ref mut axis) = self.x_axis {
            axis.mayor_tick_interval = Some(mayor_tick_interval);
        }
        if let Some(ref mut axis) = self.y_axis {
            axis.mayor_tick_interval = Some(mayor_tick_interval);
        }
        self
    }

    pub fn with_mayor_tick_interval_x(mut self, mayor_tick_interval: Tick) -> CoordinateSystem<D> {
        if let Some(ref mut axis) = self.x_axis {
            axis.mayor_tick_interval = Some(mayor_tick_interval);
        }
        self
    }

    pub fn with_mayor_tick_interval_y(mut self, mayor_tick_interval: Tick) -> CoordinateSystem<D> {
        if let Some(ref mut axis) = self.y_axis {
            axis.mayor_tick_interval = Some(mayor_tick_interval);
        }
        self
    }

    pub fn with_x_axis_placement(mut self, placment: Placement) -> CoordinateSystem<D> {
        if let Some(ref mut axis) = self.x_axis {
            axis.placement = placment;
        }
        self
    }

    pub fn with_y_axis_placement(mut self, placment: Placement) -> CoordinateSystem<D> {
        if let Some(ref mut axis) = self.y_axis {
            axis.placement = placment;
        }
        self
    }
}

impl<D> Default for CoordinateSystem<D> {
    fn default() -> Self {
        CoordinateSystem::new()
    }
}

impl<D> Drawable for CoordinateSystem<D> {
    type DrawData = D;

    fn draw(&mut self, handle: &mut CanvasHandle, _draw_data: &D) {
        let color = if handle.dark_mode() {
            Color32::WHITE
        } else {
            Color32::BLACK
        };

        if let Some(ref mut axis) = self.x_axis {
            axis.draw(handle, color, Kind::X);
        }
        if let Some(ref mut axis) = self.y_axis {
            axis.draw(handle, color, Kind::Y);
        }
    }

    fn get_cutout(&mut self, _draw_data: &D) -> Rect {
        //Coordinate System is an overlay so there is no cutout
        Rect::NOTHING
    }
}

#[derive(Debug, Clone, Default)]
pub struct Axis {
    ///the interval for the minor ticks None for no minor ticks
    ///todo unimplmented
    minor_tick_interval: Option<Tick>,

    ///the interval for the mayor ticks None for no mayor ticks
    mayor_tick_interval: Option<Tick>,

    ///draw thin lines at the mayor tick interval
    ///has only affect if mayor_tick_interval is Some
    ///todo unimplmented
    lines: bool,

    ///labeling for the axis
    ///todo unimplmented
    label: String,

    ///the number of mayor ticks to do None for infinity
    ///todo unimplmented
    length: Option<usize>,

    ///positon of the axis
    placement: Placement,
}

impl Axis {
    fn draw(&self, handle: &mut CanvasHandle, color: Color32, kind: Kind) {
        let bounding_box = handle.bounding_box();
        //draw the line
        let points = self.get_line_points(handle, bounding_box, kind);
        handle.line_segment(points, (THICK_LINE_WIDTH, color));

        if let Some(mayor_tick_interval) = self.mayor_tick_interval {
            let font_id = FontId {
                size: 16.0,
                family: FontFamily::Monospace,
            };

            let draw_region = handle.get_draw_region_in_canvas_space();
            let draw_space = match kind {
                Kind::X => draw_region.width(),
                Kind::Y => draw_region.height(),
            };
            Axis::draw_mayor_ticks(
                handle,
                color,
                font_id,
                points,
                mayor_tick_interval.get_absolute_tick(draw_space),
                kind,
            );
        }
        //todo draw the rest
    }

    fn draw_mayor_ticks(
        handle: &mut CanvasHandle,
        color: Color32,
        font_id: FontId,
        axis_line: (Position, Position),
        mayor_tick_interval: f32,
        kind: Kind,
    ) {
        let (start, end) = axis_line;
        let start_on_canvas = handle.convert_to_canvas_space(start).get_raw_pos();
        let end_on_canvas = handle.convert_to_canvas_space(end).get_raw_pos();

        use Kind::{X, Y};
        use Position::Canvas;
        match kind {
            X => {
                let ticks_left_out_of_bounds = start_on_canvas.x / mayor_tick_interval;
                let first_tick_x = if ticks_left_out_of_bounds > 0.0 {
                    ticks_left_out_of_bounds.ceil() * mayor_tick_interval
                } else {
                    ticks_left_out_of_bounds.trunc() * mayor_tick_interval
                };
                let mut tick_x = first_tick_x;
                while tick_x <= end_on_canvas.x {
                    let pos = Canvas(Pos2 {
                        x: tick_x,
                        y: start_on_canvas.y,
                    });
                    Axis::draw_mayor_tick(handle, color, font_id.clone(), pos, kind);
                    tick_x += mayor_tick_interval;
                }
            }
            Y => {
                let ticks_bottom_out_of_bounds = start_on_canvas.y / mayor_tick_interval;
                let first_tick_y = if ticks_bottom_out_of_bounds > 0.0 {
                    ticks_bottom_out_of_bounds.ceil() * mayor_tick_interval
                } else {
                    ticks_bottom_out_of_bounds.trunc() * mayor_tick_interval
                };
                let mut tick_y = first_tick_y;
                while tick_y <= end_on_canvas.y {
                    let pos = Canvas(Pos2 {
                        x: start_on_canvas.x,
                        y: tick_y,
                    });
                    Axis::draw_mayor_tick(handle, color, font_id.clone(), pos, kind);
                    tick_y += mayor_tick_interval;
                }
            }
        }
    }

    fn draw_mayor_tick(
        handle: &mut CanvasHandle,
        color: Color32,
        font_id: FontId,
        pos: Position,
        kind: Kind,
    ) {
        use Position::Overlay;
        let overlay_pos = handle.convert_to_overlay_space(pos);
        let canvas_pos = handle.convert_to_canvas_space(pos);
        let pos = overlay_pos.get_raw_pos();
        use Kind::{X, Y};
        match kind {
            X => {
                let pos_bottom = Overlay(Pos2 {
                    x: pos.x,
                    y: pos.y - MAYOR_TICK_STROKE_LENGHT / 2.0,
                });
                let pos_top = Overlay(Pos2 {
                    x: pos.x,
                    y: pos.y + MAYOR_TICK_STROKE_LENGHT / 2.0,
                });
                handle.line_segment((pos_bottom, pos_top), (THICK_LINE_WIDTH, color));

                let text = Self::print_float(canvas_pos.get_raw_pos().x);
                let size = handle.text_size(&text, font_id.clone());
                let text_pos = Overlay(Pos2 {
                    x: pos.x,
                    //subtract the 2.0 for a bit of space between the mayor tick strock and the number text
                    y: pos.y - size.y() - MAYOR_TICK_STROKE_LENGHT / 2.0 - 2.0,
                });
                handle.text(text_pos, Align2::CENTER_BOTTOM, text, font_id, color)
            }
            Y => {
                let pos_left = Overlay(Pos2 {
                    x: pos.x - MAYOR_TICK_STROKE_LENGHT / 2.0,
                    y: pos.y,
                });
                let pos_right = Overlay(Pos2 {
                    x: pos.x + MAYOR_TICK_STROKE_LENGHT / 2.0,
                    y: pos.y,
                });
                handle.line_segment((pos_left, pos_right), (THICK_LINE_WIDTH, color));

                let text = Self::print_float(canvas_pos.get_raw_pos().y);
                let size = handle.text_size(&text, font_id.clone());
                let text_pos = Overlay(Pos2 {
                    //subtract the 2.0 for a bit of space between the mayor tick strock and the number text
                    x: pos.x - size.x() - MAYOR_TICK_STROKE_LENGHT / 2.0 - 2.0,
                    y: pos.y,
                });
                handle.text(text_pos, Align2::LEFT_CENTER, text, font_id, color)
            }
        }
    }

    fn print_float(float: f32) -> String {
        let sign = if float < 0.0 { "-" } else { "" };
        let float = float.abs();
        if float >= 10_000.0 || (0.000001..=0.0001).contains(&float) {
            let log_10 = float.log10().floor();
            let new_float = float / 10.0_f32.powf(log_10);
            format!("{sign}{new_float:.2}e{log_10}")
        } else if float < 0.000001 {
            "0".to_string()
        } else {
            let string = format!("{sign}{float:.6}");
            let string: String = string.chars().take(5).collect();
            string.trim_end_matches('.').into()
        }
    }

    fn get_line_points(
        &self,
        handle: &CanvasHandle,
        bounding_box: Rectangle,
        kind: Kind,
    ) -> (Position, Position) {
        use Placement::{Canvas, Overlay};
        match &self.placement {
            Overlay(alignment) => {
                Axis::get_base_line_points_for_overlay_placement(bounding_box, *alignment, kind)
            }

            Canvas(axis_section) => Axis::get_base_line_points_for_canvas_placement(
                handle,
                bounding_box,
                *axis_section,
                kind,
            ),
        }
    }

    fn get_base_line_points_for_overlay_placement(
        bounding_box: Rectangle,
        alignment: Alignment,
        kind: Kind,
    ) -> (Position, Position) {
        use Alignment::{Center, LeftOrBottom, RightOrTop};
        use Kind::{X, Y};
        use Position::Overlay;
        let bottom = bounding_box.bottom();
        let top = bounding_box.top();
        let left = bounding_box.left();
        let right = bounding_box.right();
        match kind {
            X => {
                let y = match alignment {
                    LeftOrBottom(padding) => bottom + padding,
                    RightOrTop(padding) => top - padding,
                    Center => (bottom + top) / 2.0,
                };
                (Overlay((left, y).into()), Overlay((right, y).into()))
            }
            Y => {
                let x = match alignment {
                    LeftOrBottom(padding) => left + padding,
                    RightOrTop(padding) => right - padding,
                    Center => (left + right) / 2.0,
                };
                (Overlay((x, bottom).into()), Overlay((x, top).into()))
            }
        }
    }

    fn get_base_line_points_for_canvas_placement(
        handle: &CanvasHandle,
        bounding_box: Rectangle,
        axis_section: f32,
        kind: Kind,
    ) -> (Position, Position) {
        use Alignment::{LeftOrBottom, RightOrTop};
        use Kind::{X, Y};
        use Position::{Canvas, Overlay};

        let min = Overlay(bounding_box.min().into());
        let max = Overlay(bounding_box.max().into());

        let left_bottom = handle.convert_to_canvas_space(min).get_raw_pos();
        let right_top = handle.convert_to_canvas_space(max).get_raw_pos();

        let bottom = left_bottom.y;
        let left = left_bottom.x;
        let top = right_top.y;
        let right = right_top.x;

        let inner_box = bounding_box.shrink(DEFAULT_PADDING);

        let inner_min = Overlay(inner_box.min().into());
        let inner_max = Overlay(inner_box.max().into());

        let inner_left_bottom = handle.convert_to_canvas_space(inner_min).get_raw_pos();
        let inner_right_top = handle.convert_to_canvas_space(inner_max).get_raw_pos();

        let inner_bottom = inner_left_bottom.y;
        let inner_left = inner_left_bottom.x;
        let inner_top = inner_right_top.y;
        let inner_right = inner_right_top.x;

        match kind {
            Y => {
                if inner_left > axis_section {
                    Axis::get_base_line_points_for_overlay_placement(
                        bounding_box,
                        LeftOrBottom(DEFAULT_PADDING),
                        kind,
                    )
                } else if inner_right < axis_section {
                    Axis::get_base_line_points_for_overlay_placement(
                        bounding_box,
                        RightOrTop(DEFAULT_PADDING),
                        kind,
                    )
                } else {
                    (
                        Canvas((axis_section, bottom).into()),
                        Canvas((axis_section, top).into()),
                    )
                }
            }
            X => {
                if inner_bottom > axis_section {
                    Axis::get_base_line_points_for_overlay_placement(
                        bounding_box,
                        LeftOrBottom(DEFAULT_PADDING),
                        kind,
                    )
                } else if inner_top < axis_section {
                    Axis::get_base_line_points_for_overlay_placement(
                        bounding_box,
                        RightOrTop(DEFAULT_PADDING),
                        kind,
                    )
                } else {
                    (
                        Canvas((left, axis_section).into()),
                        Canvas((right, axis_section).into()),
                    )
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
pub enum Placement {
    ///Axis is fixed in the overlay
    Overlay(Alignment),

    ///Axis is fixed in the canvas at a given x or y position depending on the axis
    ///this means that it can be draged
    Canvas(f32),
}

impl Default for Placement {
    fn default() -> Self {
        Placement::Overlay(Alignment::LeftOrBottom(DEFAULT_PADDING))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    ///Left with padding
    LeftOrBottom(f32),
    ///Right with padding
    RightOrTop(f32),
    Center,
}

#[derive(Debug, Clone, Copy)]
pub enum Tick {
    Absolute(f32),
    ///try to print the amount of ticks
    Automatic(u8),
}

impl Tick {
    ///get the tick distance
    ///draw_space is the width or height of the axis
    ///depending on the Axis Kind (X or Y)
    fn get_absolute_tick(self, draw_space: f32) -> f32 {
        match self {
            Tick::Absolute(tick) => tick,
            Tick::Automatic(wanted_num_ticks) => {
                let mut draw_space = draw_space.abs() as f64;

                let mut tick_shrink_factor = 1.0;
                //todo is 1000 the right value here?
                while draw_space < 1000.0 * wanted_num_ticks as f64 {
                    draw_space *= 10.0;
                    tick_shrink_factor /= 10.0;
                }

                let best_tick = self.get_best_tick_from_big(draw_space as u64, wanted_num_ticks);

                (best_tick as f64 * tick_shrink_factor) as f32
            }
        }
    }

    fn get_best_tick_from_big(&self, draw_space: u64, wanted_num_ticks: u8) -> u64 {
        let min_num_ticks = min(wanted_num_ticks, MIN_NUMBER_OF_TICKS);

        let tick_options = [1, 2, 5, 25];

        let mut best_tick = 1;
        let mut num_ticks_with_best_tick = draw_space / best_tick;

        let mut rest_draw_space = draw_space;
        let mut growing_tick = 1;
        while rest_draw_space != 0 {
            for tick_option in tick_options {
                let new_num_ticks = rest_draw_space / tick_option;

                let best_diff = (wanted_num_ticks as u64).abs_diff(num_ticks_with_best_tick);
                let new_diff = (wanted_num_ticks as u64).abs_diff(new_num_ticks);
                if new_num_ticks >= min_num_ticks as u64 && new_diff < best_diff {
                    best_tick = growing_tick * tick_option;
                    num_ticks_with_best_tick = new_num_ticks;
                }
            }

            rest_draw_space /= 10;
            growing_tick *= 10;
        }
        best_tick
    }
}
