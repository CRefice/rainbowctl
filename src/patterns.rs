use crate::{Color, Lights, LightsHandle};

use anyhow::Result;
use palette::Hue;
use std::convert::TryInto;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{watch::Receiver, Mutex};
use tokio::time::{self, Duration};

pub async fn off(data: PatternData) {
    data.lights.lock().await.off();
}

pub async fn solid(mut data: PatternData) {
    let mut lights = data.lights.lock().await;
    while let Some(color) = data.colors.recv().await {
        lights.fill(color);
    }
}

pub async fn pulse(mut data: PatternData) {
    const FRAMERATE: u64 = 60;
    const FRAME_WAIT: Duration = Duration::from_millis(1000 / FRAMERATE);
    const RESOLUTION: u16 = 100;

    let mut color = data.default_color();
    let mut lights = data.lights.lock().await;

    let values = 0..RESOLUTION;
    let values = values.clone().chain(values.rev());
    // Add another few zeroes to lengthen the amount of time when light is off
    let values = std::iter::repeat(0).take(3).chain(values).cycle();
    let mut values = values.map(|x| f32::from(x) / (2.0 * f32::from(RESOLUTION)));

    let mut value = values.next().unwrap();
    let mut interval = time::interval(FRAME_WAIT);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                value = values.next().unwrap();
            }
            Some(new_color) = data.colors.recv() => {
                color = new_color;
            }
        }
        lights.render(std::iter::repeat({
            let mut color = color;
            color.lightness = value;
            color
        }))
    }
}

pub async fn rainbow(data: PatternData) {
    const FRAMERATE: u64 = 60;
    const FRAME_WAIT: Duration = Duration::from_millis(1000 / FRAMERATE);
    const RESOLUTION: u16 = 1000;

    let color = data.default_color();
    let mut lights = data.lights.lock().await;
    let mut interval = time::interval(FRAME_WAIT);
    for hue in (0..RESOLUTION).cycle() {
        let hue = 360.0 * f32::from(hue) / f32::from(RESOLUTION);
        interval.tick().await;
        lights.fill(color.with_hue(hue));
    }
}

pub async fn chase_generic(mut data: PatternData, mut direction: impl Iterator<Item = usize>) {
    const FRAMERATE: u64 = 20;
    const FRAME_WAIT: Duration = Duration::from_millis(1000 / FRAMERATE);
    const FALLOFF: f32 = 0.3;

    let mut color = data.default_color();
    let mut lights = data.lights.lock().await;

    let mut values = vec![0f32; lights.led_count()];
    let mut interval = time::interval(FRAME_WAIT);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                values.iter_mut().for_each(|value| *value *= FALLOFF);
                values[direction.next().unwrap()] = color.lightness;
            }
            Some(new_color) = data.colors.recv() => {
                color = new_color;
            }
        }
        lights.render(values.iter().map(|value| {
            let mut color = color;
            color.lightness = *value;
            color
        }))
    }
}

pub async fn chase(data: PatternData) {
    let range = 0..data.led_count().await;
    let cycle = range.clone().chain(range.rev()).cycle();
    chase_generic(data, cycle).await;
}

pub async fn chase_loop(data: PatternData) {
    let cycle = (0..data.led_count().await).cycle();
    chase_generic(data, cycle).await;
}

pub async fn equalizer(mut data: PatternData) {
    const PORT: u16 = 9999;

    let buf_size = 4 + data.led_count().await;
    let mut socket = UdpSocket::bind(("0.0.0.0", PORT)).await.unwrap();

    let mut color = data.default_color();
    let mut buf: Vec<u8> = vec![0; buf_size];

    let mut values: Vec<u8> = Vec::new();

    let mut frame = 0u32;

    let mut lights = data.lights.lock().await;
    loop {
        tokio::select! {
            _ = socket.recv_from(&mut buf) => {
                let counter = u32::from_le_bytes(buf[..4].try_into().unwrap());
                if frame < counter {
                    frame = counter;
                    values.clear();
                    values.extend_from_slice(&buf[4..]);
                }
            }
            Some(new_color) = data.colors.recv() => {
                color = new_color;
            }
        }
        lights.render(values.iter().map(|value| {
            let value = f32::from(*value) / (2.0 * f32::from(u8::MAX));
            let mut color = color;
            color.lightness = value;
            color
        }))
    }
}

#[derive(Clone)]
pub struct PatternData {
    pub lights: LightsHandle,
    pub colors: Receiver<Color>,
}

impl PatternData {
    pub fn new(colors: Receiver<Color>) -> Result<Self> {
        let lights = Arc::new(Mutex::new(Lights::new()?));
        Ok(PatternData { lights, colors })
    }

    pub fn default_color(&self) -> Color {
        *self.colors.borrow()
    }

    pub async fn led_count(&self) -> usize {
        self.lights.lock().await.led_count()
    }
}
