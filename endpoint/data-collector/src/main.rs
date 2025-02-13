use std::{
    collections::HashMap,
    net::{Ipv6Addr, SocketAddr},
};

use futures::TryStreamExt as _;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyveos_sdk::Connection;
use prometheus::{core::Collector, Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum MetricType {
    Counter,
    Gauge,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportedMetric {
    name: String,
    value: f64,
    metric_type: MetricType,
}

#[derive(Debug, Clone)]
enum Metric {
    Counter(prometheus::Counter),
    Gauge(prometheus::Gauge),
}

impl From<Metric> for Box<dyn Collector> {
    fn from(value: Metric) -> Self {
        match value {
            Metric::Counter(c) => Box::new(c),
            Metric::Gauge(g) => Box::new(g),
        }
    }
}

async fn serve_req(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", encoder.format_type())
        .body(Full::new(Bytes::from(buffer)))
        .unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = SocketAddr::new(std::net::IpAddr::V6(Ipv6Addr::UNSPECIFIED), 8080);
    let listener = TcpListener::bind(addr).await?;
    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service_fn(serve_req))
                    .await
                {
                    eprintln!("Error: {}", e);
                }
            });
        }
    });

    let connection = Connection::new().await?;

    let mut stream = connection.pub_sub().subscribe("export_data").await?;

    let mut metrics: HashMap<String, Metric> = HashMap::new();

    while let Some(msg) = stream.try_next().await? {
        let exported_metric: ExportedMetric = serde_json::from_slice(&msg.message.data[..])?;
        let metric = metrics
            .entry(exported_metric.name.clone())
            .or_insert_with_key(|name| {
                let metric = match exported_metric.metric_type {
                    MetricType::Counter => {
                        Metric::Counter(prometheus::Counter::new(name, "counter").unwrap())
                    }
                    MetricType::Gauge => {
                        Metric::Gauge(prometheus::Gauge::new(name, "gauge").unwrap())
                    }
                };
                let _ = prometheus::register(metric.clone().into());
                metric
            });
        match (metric, exported_metric.metric_type) {
            (Metric::Counter(c), MetricType::Counter) => c.inc_by(exported_metric.value),
            (Metric::Gauge(g), MetricType::Gauge) => g.set(exported_metric.value),
            _ => {
                eprintln!("Metric type mismatch");
                continue;
            }
        }
    }

    Ok(())
}
