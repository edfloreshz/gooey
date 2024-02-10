use std::time::Duration;

use figures::units::Px;
use figures::{Size, Zero};

use crate::animation::easings::{EaseInQuadradic, EaseOutQuadradic};
use crate::animation::{AnimationHandle, AnimationTarget, EasingFunction, Spawn};
use crate::context::LayoutContext;
use crate::value::{Dynamic, IntoDynamic, Source};
use crate::widget::{MakeWidget, WidgetRef, WrappedLayout, WrapperWidget};
use crate::ConstraintLimit;

/// A widget that collapses/hides its contents based on a [`Dynamic<bool>`].
#[derive(Debug)]
pub struct Collapse {
    child: WidgetRef,
    collapse: Dynamic<bool>,
    size: Dynamic<Px>,
    collapse_animation: Option<CollapseAnimation>,
    vertical: bool,
}

impl Collapse {
    /// Returns a widget that collapses `child` vertically based on the dynamic
    /// boolean value.
    ///
    /// This widget will be collapsed when the dynamic contains `true`, and
    /// revealed when the dynamic contains `false`.
    pub fn vertical(collapse_when: impl IntoDynamic<bool>, child: impl MakeWidget) -> Self {
        Self {
            collapse: collapse_when.into_dynamic(),
            child: WidgetRef::new(child),
            size: Dynamic::default(),
            vertical: true,
            collapse_animation: None,
        }
    }

    /// Returns a widget that collapses `child` horizontally based on the
    /// dynamic boolean value.
    ///
    /// This widget will be collapsed when the dynamic contains `true`, and
    /// revealed when the dynamic contains `false`.
    pub fn horizontal(collapse_when: impl IntoDynamic<bool>, child: impl MakeWidget) -> Self {
        Self {
            collapse: collapse_when.into_dynamic(),
            child: WidgetRef::new(child),
            size: Dynamic::default(),
            vertical: false,
            collapse_animation: None,
        }
    }

    fn note_child_size(&mut self, size: Px, context: &mut LayoutContext<'_, '_, '_, '_>) {
        let (easing, target) = if self.collapse.get_tracking_invalidate(context) {
            (EasingFunction::from(EaseOutQuadradic), Px::ZERO)
        } else {
            (EasingFunction::from(EaseInQuadradic), size)
        };
        match &self.collapse_animation {
            Some(state) if state.target == target => {}
            _ => {
                // If this is our first setup, immediately give the child the
                // space they request.
                let duration = if self.collapse_animation.is_some() {
                    Duration::from_millis(250)
                } else {
                    Duration::ZERO
                };
                self.collapse_animation = Some(CollapseAnimation {
                    target,
                    _handle: self
                        .size
                        .transition_to(target)
                        .over(duration)
                        .with_easing(easing)
                        .spawn(),
                });
            }
        }
    }
}

impl WrapperWidget for Collapse {
    fn child_mut(&mut self) -> &mut WidgetRef {
        &mut self.child
    }

    fn position_child(
        &mut self,
        size: Size<Px>,
        _available_space: Size<ConstraintLimit>,
        context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> WrappedLayout {
        let clip_size = self.size.get_tracking_invalidate(context);
        if self.vertical {
            self.note_child_size(size.height, context);

            Size::new(size.width, clip_size)
        } else {
            self.note_child_size(size.width, context);

            Size::new(clip_size, size.height)
        }
        .into()
    }

    fn summarize(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Collapse")
            .field("collapse", &self.collapse)
            .field("child", &self.child)
            .finish()
    }
}

#[derive(Debug)]
struct CollapseAnimation {
    target: Px,
    _handle: AnimationHandle,
}
