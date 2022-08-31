use epaint::{emath::Align, vec2, Color32, Pos2, Rect, Shape, Stroke, Vec2};

use crate::{FontSelection, Painter, Response, Sense, Ui, WidgetText};

pub struct Decorator {
    pub outside_painter: Painter,
    pub offset_to_inside: Vec2,
    pub axis_margin: f32,
}

impl Decorator {
    /// returns true if Decorator drew the label
    pub fn add_axis_grid_label(
        &self,
        ui: &Ui,
        pos_in_gui: Pos2,
        text: impl Into<WidgetText>,
        axis_positions: &[AxisPosition; 2],
        color: Color32,
        line_stroke: Stroke,
        axis: usize,
        shapes_outer: &mut Vec<Shape>,
    ) -> bool {
        const GRID_LABEL_VALIGN: [Align; 2] = [Align::Min, Align::Center];
        const GRID_LABEL_HALIGN: [Align; 2] = [Align::Center, Align::Max];

        let axis_pos = axis_positions[axis];
        if !axis_pos.drawn_by_decorator() {
            return false;
        }

        let valign = GRID_LABEL_VALIGN[axis];
        let mut text_job = text
            .into()
            .into_text_job(ui.style(), FontSelection::Default, valign);
        text_job.job.halign = GRID_LABEL_HALIGN[axis];
        let galley = text_job.into_galley(&*ui.fonts()).galley;
        let mut text_pos = pos_in_gui;
        text_pos = text_pos + axis_pos.label_shift(axis, 5.0);
        shapes_outer.push(Shape::galley_with_color(text_pos, galley, color));

        shapes_outer.push(Shape::LineSegment {
            points: [pos_in_gui, text_pos],
            stroke: line_stroke,
        });
        true
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum AxisPosition {
    OutsideLow,
    Low,
    High,
    OutsideHigh,
    AtCross,
}

impl AxisPosition {
    pub fn needs_outside_margin(&self) -> bool {
        match self {
            AxisPosition::OutsideLow => true,
            AxisPosition::OutsideHigh => true,
            _ => false,
        }
    }

    pub fn drawn_by_decorator(&self) -> bool {
        match self {
            AxisPosition::OutsideLow => true,
            AxisPosition::OutsideHigh => true,
            _ => false,
        }
    }

    pub fn label_shift(&self, axis_idx: usize, axis_offset: f32) -> Vec2 {
        let shift = axis_offset * 0.5;
        match self {
            AxisPosition::OutsideLow => {
                if axis_idx == 0 {
                    // x axis
                    vec2(0.0, shift)
                } else {
                    // y axis
                    vec2(-shift, 0.0)
                }
            }
            AxisPosition::OutsideHigh => {
                if axis_idx == 0 {
                    // x axis
                    vec2(0.0, -shift)
                } else {
                    // y axis
                    vec2(shift, 0.0)
                }
            }
            _ => vec2(0.0, 0.0),
        }
    }
}

pub fn allocate_space_and_decorator_for_plot(
    ui: &mut Ui,
    size: Vec2,
    axis_positions: &[AxisPosition; 2],
    axis_margin: f32,
) -> (Rect, Response, Decorator) {
    let need_x_margin = axis_positions[1].needs_outside_margin();
    let need_y_margin = axis_positions[0].needs_outside_margin();
    let size_with_outside: Vec2;
    let offset_to_inside: Vec2;
    if need_x_margin || need_y_margin {
        let x_margin = if need_x_margin { axis_margin } else { 0.0 };
        let y_margin = if need_y_margin { axis_margin } else { 0.0 };
        size_with_outside = size + vec2(x_margin, y_margin);

        offset_to_inside = vec2(
            if axis_positions[1] == AxisPosition::OutsideLow {
                x_margin
            } else {
                0.0
            },
            if axis_positions[0] == AxisPosition::OutsideHigh {
                y_margin
            } else {
                0.0
            },
        );
    } else {
        size_with_outside = size;
        offset_to_inside = vec2(0.0, 0.0);
    }

    let (outside_rect, _) = ui.allocate_exact_size(size_with_outside, Sense::hover());
    let outside_painter = Painter::new(ui.ctx().clone(), ui.layer_id(), outside_rect);
    let rect = Rect::from_min_size(outside_rect.left_top() + offset_to_inside, size);
    // Allocate the space.
    let response = ui.allocate_rect(rect, Sense::drag());

    // make sure cursor is advanced correctly
    ui.advance_cursor_after_rect(outside_rect);

    (
        rect,
        response,
        Decorator {
            outside_painter,
            offset_to_inside,
            axis_margin,
        },
    )
}
