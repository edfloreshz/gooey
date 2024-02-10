use std::fmt::Debug;

use figures::units::UPx;
use figures::{Fraction, IntoSigned, Point, Rect, ScreenScale, Size, Zero};

use crate::context::{AsEventContext, EventContext, LayoutContext};
use crate::styles::{Edges, FlexibleDimension};
use crate::value::{IntoValue, Value};
use crate::widget::{MakeWidget, RootBehavior, WidgetRef, WrappedLayout, WrapperWidget};
use crate::ConstraintLimit;

/// A widget aligns its contents to its container's boundaries.
#[derive(Debug)]
pub struct Align {
    child: WidgetRef,
    edges: Value<Edges<FlexibleDimension>>,
}

impl Align {
    /// Returns a new spacing widget containing `widget`, surrounding it with
    /// `margin`.
    pub fn new(margin: impl IntoValue<Edges<FlexibleDimension>>, widget: impl MakeWidget) -> Self {
        Self {
            child: WidgetRef::new(widget),
            edges: margin.into_value(),
        }
    }

    /// Returns a new spacing widget that centers `widget` vertically and
    /// horizontally.
    pub fn centered(widget: impl MakeWidget) -> Self {
        Self::new(FlexibleDimension::Auto, widget)
    }

    /// Sets the left edge of alignment to 0 and returns self.
    #[must_use]
    pub fn align_left(mut self) -> Self {
        self.edges
            .map_mut(|mut edges| edges.left = FlexibleDimension::ZERO);
        self
    }

    /// Sets the top edge of alignment to 0 and returns self.
    #[must_use]
    pub fn align_top(mut self) -> Self {
        self.edges
            .map_mut(|mut edges| edges.top = FlexibleDimension::ZERO);
        self
    }

    /// Sets the bottom edge of alignment to 0 and returns self.
    #[must_use]
    pub fn align_bottom(mut self) -> Self {
        self.edges
            .map_mut(|mut edges| edges.bottom = FlexibleDimension::ZERO);
        self
    }

    /// Sets the right edge of alignment to 0 and returns self.
    #[must_use]
    pub fn align_right(mut self) -> Self {
        self.edges
            .map_mut(|mut edges| edges.right = FlexibleDimension::ZERO);
        self
    }

    /// Sets the left and right edges of alignment to 0 and returns self.
    #[must_use]
    pub fn fit_horizontally(mut self) -> Self {
        self.edges.map_mut(|mut edges| {
            edges.left = FlexibleDimension::ZERO;
            edges.right = FlexibleDimension::ZERO;
        });
        self
    }

    /// Sets the top and bottom edges of alignment to 0 and returns self.
    #[must_use]
    pub fn fit_vertically(mut self) -> Self {
        self.edges.map_mut(|mut edges| {
            edges.top = FlexibleDimension::ZERO;
            edges.bottom = FlexibleDimension::ZERO;
        });
        self
    }

    fn measure(
        &mut self,
        available_space: Size<ConstraintLimit>,
        context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> Layout {
        let margin = self.edges.get();
        let vertical = FrameInfo::new(context.gfx.scale(), margin.top, margin.bottom);
        let horizontal = FrameInfo::new(context.gfx.scale(), margin.left, margin.right);

        let content_available = Size::new(
            horizontal.child_constraint(available_space.width),
            vertical.child_constraint(available_space.height),
        );

        let child = self.child.mounted(&mut context.as_event_context());
        let content_size = context.for_other(&child).layout(content_available);

        let (left, right, width) = horizontal.measure(available_space.width, content_size.width);
        let (top, bottom, height) = vertical.measure(available_space.height, content_size.height);

        Layout {
            margin: Edges {
                left,
                top,
                right,
                bottom,
            },
            content: Size::new(width, height),
        }
    }
}

struct FrameInfo {
    a: Option<UPx>,
    b: Option<UPx>,
}

impl FrameInfo {
    fn new(scale: Fraction, a: FlexibleDimension, b: FlexibleDimension) -> Self {
        let a = match a {
            FlexibleDimension::Auto => None,
            FlexibleDimension::Dimension(dimension) => Some(dimension.into_upx(scale)),
        };
        let b = match b {
            FlexibleDimension::Auto => None,
            FlexibleDimension::Dimension(dimension) => Some(dimension.into_upx(scale)),
        };
        Self { a, b }
    }

    fn child_constraint(&self, available: ConstraintLimit) -> ConstraintLimit {
        match (self.a, self.b) {
            (Some(a), Some(b)) => available - (a + b),
            // If we have at least one auto-measurement, force the constraint
            // into ClippedAfter mode to make the widget attempt to size the
            // content to fit.
            (Some(one), None) | (None, Some(one)) => {
                ConstraintLimit::SizeToFit(available.max() - one)
            }
            (None, None) => ConstraintLimit::SizeToFit(available.max()),
        }
    }

    fn measure(&self, available: ConstraintLimit, content: UPx) -> (UPx, UPx, UPx) {
        match available {
            ConstraintLimit::Fill(size) => {
                let size = size.max(content);
                let remaining = size.saturating_sub(content);
                let (a, b) = match (self.a, self.b) {
                    (Some(a), Some(b)) => (a, b),
                    (Some(a), None) => (a, remaining - a),
                    (None, Some(b)) => (remaining - b, b),
                    (None, None) => {
                        let a = remaining / 2;
                        let b = remaining - a;
                        (a, b)
                    }
                };

                (a, b, size - a - b)
            }
            ConstraintLimit::SizeToFit(_) => (
                self.a.unwrap_or_default(),
                self.b.unwrap_or_default(),
                content,
            ),
        }
    }
}

impl WrapperWidget for Align {
    fn child_mut(&mut self) -> &mut WidgetRef {
        &mut self.child
    }

    fn root_behavior(&mut self, _context: &mut EventContext<'_>) -> Option<RootBehavior> {
        Some(RootBehavior::Align)
    }

    fn layout_child(
        &mut self,
        available_space: Size<ConstraintLimit>,
        context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> WrappedLayout {
        let layout = self.measure(available_space, context);

        WrappedLayout {
            child: Rect::new(
                Point::new(layout.margin.left, layout.margin.top).into_signed(),
                layout.content.into_signed(),
            ),
            size: layout.content + layout.margin.size(),
        }
    }
}

impl AsMut<WidgetRef> for Align {
    fn as_mut(&mut self) -> &mut WidgetRef {
        &mut self.child
    }
}

#[derive(Debug)]
struct Layout {
    margin: Edges<UPx>,
    content: Size<UPx>,
}
