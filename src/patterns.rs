use crate::{Color, Lights, LightsHandle};

use anyhow::Result;
use std::convert::TryInto;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{watch::Receiver, Mutex};
use tokio::time::{self, Duration};

const PI: f32 = std::f32::consts::PI;
const FRAMERATE: u16 = 60;
const FRAME_WAIT: Duration = Duration::from_millis(1000 / FRAMERATE as u64);

fn float_range(resolution: u16) -> impl Clone + DoubleEndedIterator<Item = f32> {
    (0..resolution).map(move |x| f32::from(x) / f32::from(resolution))
}

fn ping_pong_range(resolution: u16) -> impl Clone + DoubleEndedIterator<Item = f32> {
    (0..resolution)
        .chain((1..resolution - 1).rev())
        .map(move |x| f32::from(x) / f32::from(resolution))
}

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
    const PERIOD: u16 = 2;

    let mut color = data.default_color();
    let mut lights = data.lights.lock().await;

    let mut values = ping_pong_range(FRAMERATE * PERIOD)
        .cycle()
        .map(|x| f32::sin(x * PI));

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
        lights.fill(color * (value * value));
    }
}

fn sinebow(hue: f32) -> Color {
    let peak = |x: f32| {
        if x.abs().fract() <= (1.0 / 3.0) {
            let y = f32::cos(1.5 * PI * x);
            y * y
        } else {
            0.0
        }
    };
    let r = peak(hue) + peak(hue - 1.0);
    let g = peak(hue - 1.0 / 3.0);
    let b = peak(hue - 2.0 / 3.0);
    Color::new(r, g, b)
}

pub async fn rainbow(data: PatternData) {
    const PERIOD: u16 = 5;
    let mut lights = data.lights.lock().await;
    let mut interval = time::interval(FRAME_WAIT);
    for hue in float_range(FRAMERATE * PERIOD).cycle() {
        interval.tick().await;
        lights.fill(sinebow(hue));
    }
}

async fn chase_generic(mut data: PatternData, mut direction: impl Iterator<Item = usize>) {
    const PERIOD: u16 = 3;
    const FALLOFF: f32 = 0.2;

    let mut color = data.default_color();
    let mut lights = data.lights.lock().await;
    let led_count = lights.led_count() as u16;
    // 60fps, 24 lights
    // that means, 2-3 frames per step to be done in ~1s.
    // if we want to be done in longer, that's going to take 4-6 frames.
    // so the formula for the number of steps is FRAMERATE * PERIOD / num_lights
    let steps_per_led = (FRAMERATE * PERIOD) / led_count;

    let mut values = vec![0f32; led_count as usize];
    let mut interval = time::interval(FRAME_WAIT);
    let mut index = direction.next().unwrap();
    let mut step = 0;
    loop {
        tokio::select! {
            _ = interval.tick() => {
                values.iter_mut().for_each(|value| *value *= 1.0 - (FALLOFF / PERIOD as f32));
                values[index] = f32::from(step) / f32::from(steps_per_led);
                values[index] = f32::powf(values[index], 2.2);
                if step == steps_per_led {
                    index = direction.next().unwrap();
                    step = 0;
                } else {
                    step += 1;
                }
            }
            Some(new_color) = data.colors.recv() => {
                color = new_color;
            }
        }
        lights.render(values.iter().map(|&value| color * value));
    }
}

pub async fn chase(data: PatternData) {
    let count = data.led_count().await;
    let range = 0..count;
    let cycle = range.chain((1..count - 1).rev()).cycle();
    chase_generic(data, cycle).await;
}

pub async fn chase_loop(data: PatternData) {
    let cycle = (0..data.led_count().await).cycle();
    chase_generic(data, cycle).await;
}

pub async fn rainbow_loop(data: PatternData) {
    const PERIOD: u16 = 4;
    let count = data.led_count().await as u16;
    let pixel_hues = float_range(count);
    let mut interval = time::interval(FRAME_WAIT);
    let mut lights = data.lights.lock().await;
    for shift in float_range(FRAMERATE * PERIOD).cycle() {
        lights.render(
            pixel_hues
                .clone()
                .map(|hue| sinebow(f32::fract(hue + shift))),
        );
        interval.tick().await;
    }
}

pub async fn equalizer(mut data: PatternData) {
    const PORT: u16 = 9999;

    let buf_size = 4 + data.led_count().await;
    let mut socket = UdpSocket::bind(("0.0.0.0", PORT)).await.unwrap();

    let mut color = data.default_color();
    let mut buf: Vec<u8> = vec![0; buf_size];

    let mut frame = 0u32;
    let mut values: Vec<u8> = Vec::new();

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
        lights.render(
            values
                .iter()
                .map(|&value| color * f32::from(value) / f32::from(u8::MAX)),
        );
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
