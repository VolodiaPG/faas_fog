use std::sync::Arc;

use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::{BidDataBase, NodesDataBase};
use crate::models::AuctionStatus;
use crate::{auction, tasks, Error};
use if_chain::if_chain;
use shared_models::auction::MarketBidProposal;
use shared_models::sla::PutSla;
use shared_models::NodeId;

/// Register a SLA and starts the auctionning process
pub async fn put_sla(
    leaf_node: NodeId,
    bid_db: Arc<Mutex<BidDataBase>>,
    nodes_db: Arc<Mutex<NodesDataBase>>,
    payload: PutSla,
) -> Result<impl warp::Reply, Rejection> {
    trace!("put sla: {:?}", payload);

    let id = tasks::call_for_bids(payload.sla.clone(), bid_db.clone(), nodes_db.clone(), leaf_node).await?;

    let res;
    {
        res = bid_db.lock().await.get(&id).unwrap().clone();
    }

    let auctions_result = if let AuctionStatus::Active(bids) = &res.auction {
        auction::second_price(&payload.sla, bids)
    } else {
        None
    };

    let res = if_chain! {
        if let Some((bid, price)) = auctions_result;
        let node = nodes_db
        .lock()
        .await
        .get(&bid.node_id)
        .map(|node| node.data.clone())
        .unwrap_or_default();
        if let AuctionStatus::Active(bids) = res.auction;
        if tasks::take_offer(&node, &bid).await.is_ok();
        then {
            bid_db.lock().await.update_auction(&id, AuctionStatus::Finished(bid.clone()));

            MarketBidProposal {
                bids,
                chosen_bid: Some(bid),
                price: Some(price),
            }
        }
        else
        {
            MarketBidProposal {
                bids: Vec::new(),
                chosen_bid: None,
                price: None,
            }
        }
    };

    Ok(
        Response::builder().body(serde_json::to_string(&res).map_err(|e| {
            error!("{}", e);
            warp::reject::custom(Error::Serialization(e))
        })?),
    )
}
