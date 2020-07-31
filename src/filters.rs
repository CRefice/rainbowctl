use super::{patterns::PatternData, Color};

use futures_util::future::{Future, FutureExt, RemoteHandle};
use std::sync::{self, Arc};
use tokio::sync::watch::Sender;
use warp::Filter;

pub type PatternHandle = Arc<sync::Mutex<Option<RemoteHandle<()>>>>;
pub type ColorSenderHandle = Arc<sync::Mutex<Sender<Color>>>;

fn with<T: Clone + Send>(
    value: T,
) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || value.clone())
}

fn to_background<T: Send>(pattern: impl Future<Output = T> + Send + 'static) -> RemoteHandle<T> {
    let (task, handle) = pattern.remote_handle();
    tokio::spawn(task);
    handle
}

fn assign(pattern: RemoteHandle<()>, handle: PatternHandle) {
    *handle.lock().unwrap() = Some(pattern);
}

pub fn pattern<F, T>(
    name: &'static str,
    pattern: F,
    data: PatternData,
    remote: PatternHandle,
) -> impl Filter<Extract = ((),), Error = warp::reject::Rejection> + Clone
where
    F: Fn(PatternData) -> T + Clone + Send,
    T: Future<Output = ()> + Send + 'static,
{
    warp::path(name)
        .and(with(data))
        .map(pattern)
        .map(to_background)
        .and(with(remote))
        .map(assign)
}

pub fn color(
    sender: ColorSenderHandle,
) -> impl Filter<Extract = ((),), Error = warp::reject::Rejection> + Clone {
    warp::body::json()
        .and(with(sender))
        .map(|color: Color, sender: ColorSenderHandle| {
            sender.lock().unwrap().broadcast(color).unwrap();
        })
}
