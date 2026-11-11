use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

use super::SCREEN_W;

/// Clear menu rows to a plain background. The read callback remains in this
/// interface so existing list sources and the stable app ABI do not change.
pub(crate) fn draw_backdrop<D>(
    lcd: &mut D,
    _read: &mut dyn FnMut(u32, &mut [u8]),
    y: i32,
    h: i32,
) where
    D: DrawTarget<Color = Rgb565>,
{
    Rectangle::new(
        Point::new(0, y),
        Size::new(SCREEN_W as u32, h.max(0) as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(lcd)
    .ok();
}
