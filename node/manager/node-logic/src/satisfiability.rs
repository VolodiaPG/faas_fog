use std::collections::HashMap;
use std::str::FromStr;

use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, Client};
use kube_metrics::node::NodeMetrics;
use log::trace;
use sla::Sla;
extern crate uom;
use if_chain::if_chain;
use lazy_static::lazy_static;
use regex::Regex;
use uom::fmt::DisplayStyle::Description;
use uom::si::f64::Information;
use uom::si::information;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Inherited an error when contacting the k8s API: {0}")]
    Kube(kube::Error),
    #[error("Unable to obtain the current key: {0}")]
    MissingKey(&'static str),
    #[error("Unable to parse the quantity: {0}")]
    QuantityParsing(String),
}

#[derive(Debug)]

struct Allocatable {
    cpu: String,
    memory: Information,
}

#[derive(Debug)]

struct Usage {
    cpu: String,
    memory: Information,
}

#[derive(Debug)]
struct Metrics {
    usage: Option<Usage>,
    allocatable: Option<Allocatable>,
}

pub async fn is_satisfiable(sla: &Sla) -> Result<bool, Error> {
    // TODO: maybe consider other backend than f64 for storing values (ie passing to fixed point u64)

    let aggregated_metrics = get_k8s_metrics().await?;

    for (key, metrics) in aggregated_metrics.iter() {
        if_chain! {
            if let Some(allocatable) = &metrics.allocatable;
            if let Some(usage) = &metrics.usage;
            if allocatable.memory - usage.memory>= sla.memory;
            then
            {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

async fn get_k8s_metrics() -> Result<HashMap<String, Metrics>, Error> {
    let mut aggregated_metrics: HashMap<String, Metrics> = HashMap::new();

    let client = Client::try_default().await.map_err(Error::Kube)?;
    let nodeMetrics: Api<NodeMetrics> = Api::all(client.clone());
    let metrics = nodeMetrics
        .list(&ListParams::default())
        .await
        .map_err(Error::Kube)?;

    for metric in metrics {
        // let memory = memory.into_format_args(gibibyte, Description);
        let key = metric
            .metadata
            .name
            .ok_or(Error::MissingKey("metadata:name"))?;

        aggregated_metrics.insert(
            key,
            Metrics {
                usage: Some(Usage {
                    cpu: metric.usage.cpu.0.to_owned(),
                    memory: parse_quantity(&metric.usage.memory.0[..])?,
                }),
                allocatable: None,
            },
        );
    }

    let nodes: Api<Node> = Api::all(client.clone());
    let nodes = nodes
        .list(&ListParams::default())
        .await
        .map_err(Error::Kube)?;

    for node in nodes {
        let status = node.status.ok_or(Error::MissingKey("status"))?;
        let allocatable = status.allocatable.ok_or(Error::MissingKey("allocatable"))?;
        let key = node
            .metadata
            .name
            .ok_or(Error::MissingKey("metadata:name"))?;
        let cpu = allocatable.get("cpu").ok_or(Error::MissingKey("cpu"))?;
        let memory = allocatable
            .get("memory")
            .ok_or(Error::MissingKey("memory"))?;

        // let memory = memory.into_format_args(gibibyte, Description);
        aggregated_metrics
            .get_mut(&key)
            .ok_or(Error::MissingKey("metadata:name"))?
            .allocatable = Some(Allocatable {
            cpu: cpu.0.to_owned(),
            memory: parse_quantity(&memory.0[..])?,
        });
    }

    Ok(aggregated_metrics)
}

fn parse_quantity<T>(quantity: &str) -> Result<T, Error>
where
    T: FromStr,
{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d+)(\w+)$").unwrap();
    }

    let captures = RE
        .captures(quantity)
        .ok_or(Error::QuantityParsing(quantity.to_string()))?;

    let measure = captures
        .get(1)
        .ok_or(Error::QuantityParsing(quantity.to_string()))?;
    let unit = captures
        .get(2)
        .ok_or(Error::QuantityParsing(quantity.to_string()))?;

    let qty = format!("{} {}B", measure.as_str(), unit.as_str())
        .parse::<T>()
        .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;

    Ok(qty)
}
