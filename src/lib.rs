pub mod filters;
pub mod patterns;

pub use palette::Hsl as Color;
pub use rs_ws281x::RawColor;

use anyhow::Result;
use palette::{named, ConvertInto, Pixel as _, Srgb};
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use std::iter;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Lights(Controller);

impl Lights {
    pub fn new() -> Result<Self> {
        let controller = ControllerBuilder::new()
            .channel(
                0,
                ChannelBuilder::new()
                    .pin(18)
                    .count(24)
                    .strip_type(StripType::Ws2811Gbr)
                    .brightness(255)
                    .build(),
            )
            .build()?;
        Ok(Lights(controller))
    }

    pub fn off(&mut self) {
        self.fill(named::BLACK);
    }

    pub fn led_count(&self) -> usize {
        self.0.leds(0).len()
    }

    pub fn fill<C, T>(&mut self, color: C)
    where
        C: ConvertInto<Srgb<T>> + Clone,
        T: palette::Component,
    {
        self.render(iter::repeat(color));
    }

    pub fn render<I, C, T>(&mut self, colors: I)
    where
        I: IntoIterator<Item = C>,
        C: ConvertInto<Srgb<T>>,
        T: palette::Component,
    {
        for (led, color) in self.0.leds_mut(0).iter_mut().zip(colors) {
            let color = color.convert_into().into_format();
            let buf: [u8; 3] = color.into_raw();
            led[0..3].copy_from_slice(&buf);
        }
        self.0.render().unwrap();
    }
}

unsafe impl Send for Lights {}

pub type LightsHandle = Arc<Mutex<Lights>>;
