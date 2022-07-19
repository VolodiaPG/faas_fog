use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

use warp::{
    filters::BoxedFilter,
    http::{Response, StatusCode},
    Filter, Reply,
};

type Result<T> = std::result::Result<T, warp::Rejection>;

#[derive(Deserialize)]
struct IncomingPayload {
    address_to_call: String,
    message: String,
}

async fn handle(payload: IncomingPayload) -> Result<impl Reply> {
    if reqwest::Client::new()
        .post(payload.address_to_call)
        .body(payload.message)
        .send()
        .await
        .is_ok()
    {
        return Ok(Box::new(StatusCode::OK));
    };
    Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
}

fn json_body() -> impl Filter<Extract = (IncomingPayload,), Error = warp::Rejection> + Clone {
    // warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    warp::body::json()
}

pub fn main() -> BoxedFilter<(impl Reply,)> {
    warp::post().and(json_body()).and_then(handle).boxed()
}
