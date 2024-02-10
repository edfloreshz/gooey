//! A labeled widget with a circular indicator representing a value.
use std::fmt::Debug;

use figures::units::Lp;
use figures::{Point, ScreenScale, Size};
use kludgine::shapes::{Shape, StrokeOptions};
use kludgine::DrawableExt;

use crate::context::{GraphicsContext, LayoutContext};
use crate::styles::components::{LineHeight, OutlineColor, WidgetAccentColor};
use crate::styles::Dimension;
use crate::value::{Destination, Dynamic, DynamicReader, IntoDynamic, IntoValue, Source, Value};
use crate::widget::{MakeWidget, MakeWidgetWithTag, Widget, WidgetInstance};
use crate::widgets::button::ButtonKind;
use crate::ConstraintLimit;

/// A labeled widget with a circular indicator representing a value.
pub struct Radio<T> {
    /// The value this button represents.
    pub value: T,
    /// The state (value) of the radio.
    pub state: Dynamic<T>,
    /// The button kind to use as the basis for this radio. Radios default to
    /// [`ButtonKind::Transparent`].
    pub kind: Value<ButtonKind>,
    label: WidgetInstance,
}

impl<T> Radio<T> {
    /// Returns a new radio that sets `state` to `value` when pressed. `label`
    /// is drawn next to the radio indicator and is also clickable to select the
    /// radio.
    pub fn new(value: T, state: impl IntoDynamic<T>, label: impl MakeWidget) -> Self {
        Self {
            value,
            state: state.into_dynamic(),
            kind: Value::Constant(ButtonKind::Transparent),
            label: label.make_widget(),
        }
    }

    /// Updates the button kind to use as the basis for this radio, and
    /// returns self.
    ///
    /// Radios default to [`ButtonKind::Transparent`].
    #[must_use]
    pub fn kind(mut self, kind: impl IntoValue<ButtonKind>) -> Self {
        self.kind = kind.into_value();
        self
    }
}

impl<T> MakeWidgetWithTag for Radio<T>
where
    T: Clone + Debug + PartialEq + Send + 'static,
{
    fn make_with_tag(self, id: crate::widget::WidgetTag) -> WidgetInstance {
        RadioOrnament {
            value: self.value.clone(),
            state: self.state.create_reader(),
        }
        .and(self.label)
        .into_columns()
        .into_button()
        .on_click(move |()| {
            self.state.set(self.value.clone());
        })
        .kind(self.kind)
        .make_with_tag(id)
    }
}

#[derive(Debug)]
struct RadioOrnament<T> {
    value: T,
    state: DynamicReader<T>,
}

impl<T> Widget for RadioOrnament<T>
where
    T: Debug + PartialEq + Send + 'static,
{
    fn redraw(&mut self, context: &mut GraphicsContext<'_, '_, '_, '_>) {
        let radio_size = context
            .gfx
            .region()
            .size
            .width
            .min(context.gfx.region().size.height);
        let vertical_center = context.gfx.region().size.height / 2;

        let stroke_options = StrokeOptions::lp_wide(Lp::points(2)).into_px(context.gfx.scale());
        context.redraw_when_changed(&self.state);
        let selected = self.state.map_ref(|state| state == &self.value);
        let color = context.get(&OutlineColor);
        let radius = radio_size / 2;
        context.gfx.draw_shape(
            Shape::stroked_circle(
                radius - stroke_options.line_width / 2,
                kludgine::Origin::Center,
                stroke_options.colored(color),
            )
            .translate_by(Point::new(radius, vertical_center)),
        );
        if selected {
            let color = context.get(&WidgetAccentColor);
            context.gfx.draw_shape(
                Shape::filled_circle(
                    radius - stroke_options.line_width * 2,
                    color,
                    kludgine::Origin::Center,
                )
                .translate_by(Point::new(radius, vertical_center)),
            );
        }
    }

    fn layout(
        &mut self,
        _available_space: Size<ConstraintLimit>,
        context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> Size<figures::units::UPx> {
        let radio_size = context.get(&RadioSize).into_upx(context.gfx.scale());
        Size::squared(radio_size)
    }
}

define_components! {
    Radio {
        /// The size to render a [`Radio`] indicator.
        RadioSize(Dimension, "size", @LineHeight)
    }
}
