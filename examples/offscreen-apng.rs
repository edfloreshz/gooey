use std::time::Duration;

use cushy::animation::easings::EaseInOutSine;
use cushy::widget::MakeWidget;
use cushy::window::VirtualRecorderError;
use figures::units::Px;
use figures::{Point, Size};

#[macro_use]
mod shared;

fn ui() -> impl MakeWidget {
    "Hello World".into_button().centered()
}

fn main() -> Result<(), VirtualRecorderError> {
    let mut recorder = ui().build_recorder().size(Size::new(320, 240)).finish()?;
    let initial_point = Point::new(Px::new(140), Px::new(150));
    recorder.set_cursor_position(initial_point);
    recorder.set_cursor_visible(true);
    recorder.refresh()?;
    let mut animation = recorder.record_animated_png(60);
    animation.animate_cursor_to(
        Point::new(Px::new(160), Px::new(120)),
        Duration::from_millis(250),
        EaseInOutSine,
    )?;
    animation.wait_for(Duration::from_millis(500))?;
    animation.animate_cursor_to(initial_point, Duration::from_millis(250), EaseInOutSine)?;
    animation.wait_for(Duration::from_millis(500))?;
    animation.write_to("examples/offscreen-apng.png")
}

adapter_required_test!(main);
