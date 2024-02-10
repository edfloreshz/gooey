#![doc = include_str!("../.crate-docs.md")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

// for proc-macros
extern crate self as cushy;

#[macro_use]
mod utils;

pub mod animation;
pub mod context;
mod graphics;
mod names;
#[macro_use]
pub mod styles;
mod app;
pub mod debug;
mod tick;
mod tree;
pub mod value;
pub mod widget;
pub mod widgets;
pub mod window;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub use app::{App, Application, Cushy, Open, PendingApp, Run};
use figures::units::UPx;
use figures::{Fraction, ScreenUnit, Size, Zero};
use kludgine::app::winit::error::EventLoopError;
pub use names::Name;
pub use utils::{Lazy, WithClone};
pub use {figures, kludgine};

pub use self::graphics::Graphics;
pub use self::tick::{InputState, Tick};

/// A limit used when measuring a widget.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConstraintLimit {
    /// The widget is expected to occupy a known size.
    Fill(UPx),
    /// The widget is expected to resize itself to fit its contents, trying to
    /// stay within the size given.
    SizeToFit(UPx),
}

impl ConstraintLimit {
    /// Returns `UPx::ZERO` when sizing to fit, otherwise it returns the size
    /// being filled.
    #[must_use]
    pub fn min(self) -> UPx {
        match self {
            ConstraintLimit::Fill(v) => v,
            ConstraintLimit::SizeToFit(_) => UPx::ZERO,
        }
    }

    /// Returns the maximum measurement that will fit the constraint.
    #[must_use]
    pub fn max(self) -> UPx {
        match self {
            ConstraintLimit::Fill(v) | ConstraintLimit::SizeToFit(v) => v,
        }
    }

    /// Converts `measured` to unsigned pixels, and adjusts it according to the
    /// constraint's intentions.
    ///
    /// If this constraint is of a known size, it will return the maximum of the
    /// measured size and the constraint. If it is of an unknown size, it will
    /// return the measured size.
    pub fn fit_measured<Unit>(self, measured: Unit, scale: Fraction) -> UPx
    where
        Unit: ScreenUnit,
    {
        let measured = measured.into_upx(scale);
        match self {
            ConstraintLimit::Fill(size) => size.max(measured),
            ConstraintLimit::SizeToFit(_) => measured,
        }
    }
}

/// An extension trait for `Size<ConstraintLimit>`.
pub trait FitMeasuredSize {
    /// Returns the result of calling [`ConstraintLimit::fit_measured`] for each
    /// matching component in `self` and `measured`.
    fn fit_measured<Unit>(self, measured: Size<Unit>, scale: Fraction) -> Size<UPx>
    where
        Unit: ScreenUnit;
}

impl FitMeasuredSize for Size<ConstraintLimit> {
    fn fit_measured<Unit>(self, measured: Size<Unit>, scale: Fraction) -> Size<UPx>
    where
        Unit: ScreenUnit,
    {
        Size::new(
            self.width.fit_measured(measured.width, scale),
            self.height.fit_measured(measured.height, scale),
        )
    }
}

impl Add<UPx> for ConstraintLimit {
    type Output = Self;

    fn add(mut self, rhs: UPx) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<UPx> for ConstraintLimit {
    fn add_assign(&mut self, rhs: UPx) {
        *self = match *self {
            ConstraintLimit::Fill(px) => ConstraintLimit::Fill(px.saturating_add(rhs)),
            ConstraintLimit::SizeToFit(px) => ConstraintLimit::SizeToFit(px.saturating_add(rhs)),
        };
    }
}

impl Sub<UPx> for ConstraintLimit {
    type Output = Self;

    fn sub(mut self, rhs: UPx) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign<UPx> for ConstraintLimit {
    fn sub_assign(&mut self, rhs: UPx) {
        *self = match *self {
            ConstraintLimit::Fill(px) => ConstraintLimit::Fill(px.saturating_sub(rhs)),
            ConstraintLimit::SizeToFit(px) => ConstraintLimit::SizeToFit(px.saturating_sub(rhs)),
        };
    }
}

/// A result alias that defaults to the result type commonly used throughout
/// this crate.
pub type Result<T = (), E = EventLoopError> = std::result::Result<T, E>;

/// Counts the number of expressions passed to it.
///
/// This is used inside of Cushy macros to preallocate collections.
#[macro_export]
#[doc(hidden)]
macro_rules! count {
    ($value:expr ;) => {
        1
    };
    ($value:expr , $($remaining:expr),+ ;) => {
        1 + $crate::count!($($remaining),+ ;)
    }
}

/// Creates a [`Styles`](crate::styles::Styles) instance with the given
/// name/component pairs.
#[macro_export]
macro_rules! styles {
    () => {{
        $crate::styles::Styles::new()
    }};
    ($($component:expr => $value:expr),*) => {{
        let mut styles = $crate::styles::Styles::with_capacity($crate::count!($($value),* ;));
        $(styles.insert(&$component, $value);)*
        styles
    }};
    ($($component:expr => $value:expr),* ,) => {{
        $crate::styles!($($component => $value),*)
    }};
}

fn initialize_tracing() {
    #[cfg(feature = "tracing-output")]
    {
        use tracing::Level;
        use tracing_subscriber::filter::LevelFilter;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::EnvFilter;

        #[cfg(debug_assertions)]
        const MAX_LEVEL: Level = Level::INFO;
        #[cfg(not(debug_assertions))]
        const MAX_LEVEL: Level = Level::ERROR;

        let _result = tracing_subscriber::fmt::fmt()
            .with_max_level(MAX_LEVEL)
            .finish()
            .with(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::from_level(MAX_LEVEL).into())
                    .from_env_lossy(),
            )
            .try_init();
    }
}
