use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::NodesDataBase;
use shared_models::{NodeId};
use shared_models::node::PatchNode;
use crate::Error;

/// Register a new node in the database
// pub async fn put_nodes(
//     nodes_db: Arc<Mutex<NodesDataBase>>,
//     payload: RegisterNode,
// ) -> Result<impl warp::Reply, Rejection> {
//     trace!("put/register a new node");

//     let node = NodeRecord {
//         ip: payload.ip,
//         ..Default::default()
//     };

//     let id;

//     {
//         id = nodes_db.lock().await.insert(node);
//     }

//     Ok(Response::builder().body(id.to_string()))
// }

/// Patch part of the node data
pub async fn patch_nodes(
    id: NodeId,
    nodes_db: Arc<Mutex<NodesDataBase>>,
    payload: PatchNode,
) -> Result<impl warp::Reply, Rejection> {
    trace!("patch the node @{}", id);

    if let Some(created_at) = payload.created_at {
        nodes_db
            .lock()
            .await
            .get_mut(&id)
            .ok_or_else(|| warp::reject::custom(Error::NodeIdNotFound(id.clone())))?
            .latency
            .update(Utc::now(), created_at);
    }

    trace!("{:#?}", nodes_db.lock().await.get_mut(&id).unwrap());

    Ok(Response::builder().body(id.to_string()))
}
