use warp::Filter;

use rainbowctl::{
    filters::{self, PatternHandle},
    patterns::{self, PatternData},
    Color,
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = watch::channel(Color::new(1.0, 0.0, 0.0));
    let tx = Arc::new(Mutex::new(tx));

    let data = PatternData::new(rx)?;
    let remote = PatternHandle::default();

    let pattern = warp::path("pattern").and(warp::post()).and(
        filters::pattern("off", patterns::off, data.clone(), remote.clone())
            .or(filters::pattern(
                "solid",
                patterns::solid,
                data.clone(),
                remote.clone(),
            ))
            .or(filters::pattern(
                "pulse",
                patterns::pulse,
                data.clone(),
                remote.clone(),
            ))
            .or(filters::pattern(
                "chase",
                patterns::chase,
                data.clone(),
                remote.clone(),
            ))
            .or(filters::pattern(
                "rainbow",
                patterns::rainbow,
                data.clone(),
                remote.clone(),
            ))
            .or(filters::pattern(
                "rainbowloop",
                patterns::rainbow_loop,
                data.clone(),
                remote.clone(),
            ))
            .or(filters::pattern(
                "chaseloop",
                patterns::chase_loop,
                data.clone(),
                remote.clone(),
            ))
            .or(filters::pattern(
                "equalizer",
                patterns::equalizer,
                data.clone(),
                remote.clone(),
            )),
    );
    let color = warp::path("color").and(filters::color(tx));

    let api = warp::path("api")
        .and(pattern.or(color))
        .map(|_| warp::reply());

    let routes = api.or(warp::fs::dir("static"));
    warp::serve(routes).run(([0, 0, 0, 0], 80)).await;
    Ok(())
}
