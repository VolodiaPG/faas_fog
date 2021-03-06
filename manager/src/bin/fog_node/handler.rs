use crate::service::function_life::FunctionLife;
use crate::service::routing::Router;
use crate::{controller, NodeLife};
use manager::helper::handler::{BytesResponse, Resp};
use manager::model::domain::routing::{FunctionRoutingStack, Packet};
use manager::model::view::auction::{BidProposals, BidRequest};
use manager::model::view::node::RegisterNode;
use manager::model::view::ping::{Ping, PingResponse};
use manager::model::BidId;
use manager::respond;
use rocket::serde::json::Json;
use rocket::{get, post, put, State};
use rocket_okapi::openapi;
use std::sync::Arc;

/// Return a bid for the SLA.
#[openapi]
#[post("/bid", data = "<payload>")]
pub async fn post_bid(payload: Json<BidRequest>,
                      function: &State<Arc<dyn FunctionLife>>)
                      -> Resp<BidProposals> {
    respond!(controller::auction::bid_on(payload.0, function.inner()).await)
}

/// Second function called after [post_bid] if the bid is accepted and the transaction starts.
/// Will then proceed to provision the SLA and thus, the function.
#[openapi]
#[post("/bid/<id>")]
pub async fn post_bid_accept(id: BidId, function: &State<Arc<dyn FunctionLife>>) -> Resp {
    respond!(controller::auction::provision_from_bid(id, function.inner()).await)
}

/// Routes the request to the correct URL and node.
#[openapi]
#[post("/routing", data = "<packet>")]
pub async fn post_routing(packet: Json<Packet<'_>>,
                          router: &State<Arc<dyn Router>>)
                          -> Result<BytesResponse, manager::helper::handler::Error> {
    Ok(BytesResponse::from(
        controller::routing::post_forward_function_routing(&packet.0, router.inner()).await?,
    ))
}

/// Register a route.
#[openapi]
#[put("/routing", data = "<stack>")]
pub async fn put_routing(router: &State<Arc<dyn Router>>,
                         stack: Json<FunctionRoutingStack>)
                         -> Resp {
    respond!(controller::routing::register_route(router.inner(), stack.0).await)
}

/// Register a child node to this one
#[openapi]
#[post("/register", data = "<payload>")]
pub async fn post_register_child_node(payload: Json<RegisterNode>,
                                      router: &State<Arc<dyn NodeLife>>)
                                      -> Resp {
    respond!(controller::node::register_child_node(payload.0, router.inner()).await)
}

/// Route to compute latencies
#[openapi]
#[post("/ping", data = "<payload>")]
pub async fn post_ping(payload: Json<Ping>) -> Json<PingResponse> {
    controller::ping::ping(payload.0).await.into()
}

#[openapi]
#[get("/health")]
pub fn health() {}
