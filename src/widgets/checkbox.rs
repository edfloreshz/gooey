//! A tri-state, labelable checkbox widget.
use std::error::Error;
use std::fmt::Display;
use std::ops::Not;

use figures::units::Lp;
use figures::{Point, Rect, Round, ScreenScale, Size};
use kludgine::shapes::{PathBuilder, Shape, StrokeOptions};

use crate::context::{GraphicsContext, LayoutContext};
use crate::styles::components::{LineHeight, OutlineColor, TextColor, WidgetAccentColor};
use crate::styles::Dimension;
use crate::value::{Dynamic, DynamicReader, IntoDynamic, IntoValue, Source, Value};
use crate::widget::{MakeWidget, MakeWidgetWithTag, Widget, WidgetInstance};
use crate::widgets::button::ButtonKind;
use crate::ConstraintLimit;

/// A labeled-widget that supports three states: Checked, Unchecked, and
/// Indeterminant
pub struct Checkbox {
    /// The state (value) of the checkbox.
    pub state: Dynamic<CheckboxState>,
    /// The button kind to use as the basis for this checkbox. Checkboxes
    /// default to [`ButtonKind::Transparent`].
    pub kind: Value<ButtonKind>,
    label: WidgetInstance,
}

impl Checkbox {
    /// Returns a new checkbox that updates `state` when clicked. `label` is
    /// drawn next to the checkbox and is also clickable to toggle the checkbox.
    ///
    /// `state` can also be a `Dynamic<bool>` if there is no need to represent
    /// an indeterminant state.
    pub fn new(state: impl IntoDynamic<CheckboxState>, label: impl MakeWidget) -> Self {
        Self {
            state: state.into_dynamic(),
            kind: Value::Constant(ButtonKind::Transparent),
            label: label.make_widget(),
        }
    }

    /// Updates the button kind to use as the basis for this checkbox, and
    /// returns self.
    ///
    /// Checkboxes default to [`ButtonKind::Transparent`].
    #[must_use]
    pub fn kind(mut self, kind: impl IntoValue<ButtonKind>) -> Self {
        self.kind = kind.into_value();
        self
    }
}

impl MakeWidgetWithTag for Checkbox {
    fn make_with_tag(self, id: crate::widget::WidgetTag) -> WidgetInstance {
        CheckboxOrnament {
            value: self.state.create_reader(),
        }
        .and(self.label)
        .into_columns()
        .into_button()
        .on_click(move |()| {
            let mut value = self.state.lock();
            *value = !*value;
        })
        .kind(self.kind)
        .make_with_tag(id)
    }
}

/// The state/value of a [`Checkbox`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckboxState {
    /// The checkbox should display showing that it is neither checked or
    /// unchecked.
    ///
    /// This state is used to represent concepts such as:
    ///
    /// - States that are neither true/false, or on/off.
    /// - States that are partially true or partially on.
    Indeterminant,
    /// The checkbox should display in an unchecked/off/false state.
    Unchecked,
    /// The checkbox should display in an checked/on/true state.
    Checked,
}

impl From<bool> for CheckboxState {
    fn from(value: bool) -> Self {
        if value {
            Self::Checked
        } else {
            Self::Unchecked
        }
    }
}

impl From<CheckboxState> for Option<bool> {
    fn from(value: CheckboxState) -> Self {
        match value {
            CheckboxState::Indeterminant => None,
            CheckboxState::Unchecked => Some(false),
            CheckboxState::Checked => Some(true),
        }
    }
}

impl From<Option<bool>> for CheckboxState {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => CheckboxState::Checked,
            Some(false) => CheckboxState::Unchecked,
            None => CheckboxState::Indeterminant,
        }
    }
}

impl TryFrom<CheckboxState> for bool {
    type Error = CheckboxToBoolError;

    fn try_from(value: CheckboxState) -> Result<Self, Self::Error> {
        match value {
            CheckboxState::Checked => Ok(true),
            CheckboxState::Unchecked => Ok(false),
            CheckboxState::Indeterminant => Err(CheckboxToBoolError),
        }
    }
}

impl Not for CheckboxState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Indeterminant | Self::Unchecked => Self::Checked,
            Self::Checked => Self::Unchecked,
        }
    }
}

impl IntoDynamic<CheckboxState> for Dynamic<bool> {
    fn into_dynamic(self) -> Dynamic<CheckboxState> {
        self.linked(
            |bool| CheckboxState::from(*bool),
            |tri_state: &CheckboxState| bool::try_from(*tri_state).ok(),
        )
    }
}

impl IntoDynamic<CheckboxState> for Dynamic<Option<bool>> {
    fn into_dynamic(self) -> Dynamic<CheckboxState> {
        self.linked(
            |bool| CheckboxState::from(*bool),
            |tri_state: &CheckboxState| bool::try_from(*tri_state).ok(),
        )
    }
}

/// An [`CheckboxState::Indeterminant`] was encountered when converting to a
/// `bool`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CheckboxToBoolError;

impl Display for CheckboxToBoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CheckboxState was Indeterminant")
    }
}

impl Error for CheckboxToBoolError {}

#[derive(Debug)]
struct CheckboxOrnament {
    value: DynamicReader<CheckboxState>,
}

impl Widget for CheckboxOrnament {
    fn redraw(&mut self, context: &mut GraphicsContext<'_, '_, '_, '_>) {
        let checkbox_size = context
            .gfx
            .region()
            .size
            .width
            .min(context.gfx.region().size.height);

        let stroke_options =
            StrokeOptions::px_wide(Lp::points(2).into_px(context.gfx.scale()).round());

        let half_line = stroke_options.line_width / 2;

        let checkbox_rect = Rect::new(
            Point::new(
                half_line,
                (context.gfx.region().size.height - checkbox_size) / 2 + half_line,
            ),
            Size::squared(checkbox_size - stroke_options.line_width),
        );

        match self.value.get_tracking_redraw(context) {
            state @ (CheckboxState::Checked | CheckboxState::Indeterminant) => {
                let color = context.get(&WidgetAccentColor);
                context
                    .gfx
                    .draw_shape(&Shape::filled_rect(checkbox_rect, color));
                let icon_area = checkbox_rect.inset(Lp::points(3).into_px(context.gfx.scale()));
                let text_color = context.get(&TextColor);
                let center = icon_area.origin + icon_area.size / 2;
                if matches!(state, CheckboxState::Checked) {
                    context.gfx.draw_shape(
                        &PathBuilder::new(Point::new(icon_area.origin.x, center.y))
                            .line_to(Point::new(
                                icon_area.origin.x + icon_area.size.width / 4,
                                icon_area.origin.y + icon_area.size.height * 3 / 4,
                            ))
                            .line_to(Point::new(
                                icon_area.origin.x + icon_area.size.width,
                                icon_area.origin.y,
                            ))
                            .build()
                            .stroke(stroke_options.colored(text_color)),
                    );
                } else {
                    context.gfx.draw_shape(
                        &PathBuilder::new(Point::new(icon_area.origin.x, center.y))
                            .line_to(Point::new(
                                icon_area.origin.x + icon_area.size.width,
                                center.y,
                            ))
                            .build()
                            .stroke(stroke_options.colored(text_color)),
                    );
                }
            }
            CheckboxState::Unchecked => {
                let color = context.get(&OutlineColor);
                context.gfx.draw_shape(&Shape::stroked_rect(
                    checkbox_rect,
                    stroke_options.colored(color),
                ));
            }
        }
    }

    fn layout(
        &mut self,
        _available_space: Size<ConstraintLimit>,
        context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> Size<figures::units::UPx> {
        let checkbox_size = context.get(&CheckboxSize).into_upx(context.gfx.scale());
        Size::squared(checkbox_size)
    }
}

/// A value that can be used as a checkbox.
pub trait Checkable: IntoDynamic<CheckboxState> + Sized {
    /// Returns a new checkbox using `self` as the value and `label`.
    fn into_checkbox(self, label: impl MakeWidget) -> Checkbox {
        Checkbox::new(self.into_dynamic(), label)
    }

    /// Returns a new checkbox using `self` as the value and `label`.
    fn to_checkbox(&self, label: impl MakeWidget) -> Checkbox
    where
        Self: Clone,
    {
        self.clone().into_checkbox(label)
    }
}

impl<T> Checkable for T where T: IntoDynamic<CheckboxState> {}

define_components! {
    Checkbox {
        /// The size to render a [`Checkbox`] indicator.
        CheckboxSize(Dimension, "size", @LineHeight)
    }
}
