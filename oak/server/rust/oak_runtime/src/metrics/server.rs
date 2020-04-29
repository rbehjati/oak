//
// Copyright 2020 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use hyper::{header::CONTENT_TYPE, Body, Request, Response, Server, service::make_service_fn, service::service_fn};
use log::info;
use prometheus::{Encoder, TextEncoder};
use std::{net::SocketAddr, sync::Arc};

use crate::Runtime;

#[derive(Debug)]
enum MetricsServerError {
    EncodingError(String),
    ResponseError(String),
}

impl std::fmt::Display for MetricsServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MetricsServerError::EncodingError(msg) => write!(f, "Metrics server error: {}", msg),
            MetricsServerError::ResponseError(msg) => write!(f, "Metrics server error: {}", msg),
        }
    }
}

impl std::error::Error for MetricsServerError {}

async fn serve_metrics(
    runtime: Arc<Runtime>,
    _req: Request<Body>,
) -> Result<Response<Body>, MetricsServerError> {
    let encoder = TextEncoder::new();
    let metric_families = &runtime.gather_metrics();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).map_err(|e| {
        MetricsServerError::EncodingError(format!("Could not encode metrics data: {}", e))
    })?;

    info!("Metrics size: {}", buffer.len());

    Response::builder()
        .status(http::StatusCode::OK)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .map_err(|e| {
            MetricsServerError::ResponseError(format!("Could not build the response: {}", e))
        })
}

async fn make_server(runtime: Arc<Runtime>, port: u16) -> Result<(), hyper::error::Error> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // A `Service` is needed for every connection, so this
    // creates one from the `process_metrics` function.
    let make_service = make_service_fn(move |_| {
        let runtime = runtime.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let runtime = runtime.clone();
                async move { serve_metrics(runtime, req).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    info!(
        "{:?}: Started metrics server on port {:?}",
        std::thread::current().id(),
        port
    );

    // Run this server for... forever!
    server.await
}

pub fn start_metrics_server(
    port: u16,
    runtime: Arc<Runtime>,
    notify_receiver: tokio::sync::oneshot::Receiver<()>,
) {
    info!("********* In metrics::server::start_metrics_server.");
    let mut tokio_runtime = tokio::runtime::Runtime::new().expect("Couldn't create Tokio runtime");
    tokio_runtime.block_on(futures::future::select(
        Box::pin(make_server(runtime, port)),
        notify_receiver,
    ));
}
