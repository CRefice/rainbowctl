pub mod filters;
pub mod patterns;

pub use palette::LinSrgb as Color;
pub use rs_ws281x::RawColor;

use anyhow::Result;
use palette::{Pixel as _, Srgb};
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
                    .brightness(128)
                    .build(),
            )
            .build()?;
        Ok(Lights(controller))
    }

    pub fn off(&mut self) {
        self.fill(Srgb::new(0.0, 0.0, 0.0).into_linear());
    }

    pub fn led_count(&self) -> usize {
        self.0.leds(0).len()
    }

    pub fn fill(&mut self, color: impl Into<Color>) {
        self.render(iter::repeat(color.into()));
    }

    pub fn render<I>(&mut self, colors: I)
    where
        I: IntoIterator<Item = Color>,
    {
        for (led, color) in self.0.leds_mut(0).iter_mut().zip(colors) {
            let buf: [u8; 3] = Srgb::from_linear(color).into_format().into_raw();
            led[0..3].copy_from_slice(&buf);
        }
        self.0.render().unwrap();
    }
}

unsafe impl Send for Lights {}

pub type LightsHandle = Arc<Mutex<Lights>>;
