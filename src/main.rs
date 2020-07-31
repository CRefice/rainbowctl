use warp::Filter;

use palette::Hsl;
use rainbowctl::{
    filters::{self, PatternHandle},
    patterns::{self, PatternData},
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

/*
macro_rules! pattern { ($p:ident) => {
        filters::pattern(
            std::stringify!($p),
            patterns::$p,
            lights.clone(),
            colors.clone(),
            remote.clone(),
        )
    };
}
*/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = watch::channel(Hsl::new(0.0, 1.0, 0.5));
    let tx = Arc::new(Mutex::new(tx));

    let data = PatternData::new(rx)?;
    let remote = PatternHandle::default();

    let off = filters::pattern("off", patterns::off, data.clone(), remote.clone());
    let solid = filters::pattern("solid", patterns::solid, data.clone(), remote.clone());
    let pulse = filters::pattern("pulse", patterns::pulse, data.clone(), remote.clone());
    let rainbow = filters::pattern("rainbow", patterns::rainbow, data.clone(), remote.clone());
    let chase = filters::pattern("chase", patterns::chase, data.clone(), remote.clone());
    let chase_loop = filters::pattern(
        "chaseloop",
        patterns::chase_loop,
        data.clone(),
        remote.clone(),
    );
    let equalizer = filters::pattern(
        "equalizer",
        patterns::equalizer,
        data.clone(),
        remote.clone(),
    );

    let pattern = warp::path("pattern").and(warp::post()).and(
        off.or(solid)
            .or(pulse)
            .or(rainbow)
            .or(chase)
            .or(chase_loop)
            .or(equalizer),
    );
    let color = warp::path("color").and(filters::color(tx));

    let api = warp::path("api")
        .and(pattern.or(color))
        .map(|_| warp::reply());

    let routes = api.or(warp::fs::dir("static"));
    warp::serve(routes).run(([0, 0, 0, 0], 80)).await;
    Ok(())
}
