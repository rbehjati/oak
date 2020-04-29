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

use prometheus::{Histogram, HistogramOpts, HistogramTimer, IntCounter, IntGauge, Opts, Registry, proto::MetricFamily};

pub mod server;

/// Struct that collects all the metrics in one place
pub struct Metrics {
    pub grpc_request_duration: Histogram,
    pub grpc_requests_total: IntCounter,
    pub grpc_response_size: Histogram,
    pub runtime_nodes_count: IntGauge,
    registry: Registry,
}

// TODO(#899): For testability implement a trait with methods for updating the metrics.
// TODO(#899): Instead of using a global Registry, the Runtime should instantiate and manage the
// Registry
impl Metrics {
    pub fn new() -> Self {
        let tmp_registry = Registry::new();
        Self {
            grpc_request_duration: Self::register(
                &tmp_registry,
                Self::histogram(
                    "grpc_request_duration_seconds",
                    "The gRPC request latencies in seconds.",
                ),
            ),

            grpc_requests_total: Self::register(
                &tmp_registry,
                Self::int_counter(
                    "grpc_requests_total",
                    "Total number of gRPC requests received.",
                ),
            ),

            grpc_response_size: Self::register(
                &tmp_registry,
                Self::histogram(
                    "grpc_response_size_bytes",
                    "The gRPC response sizes in bytes.",
                ),
            ),
            runtime_nodes_count: Self::register(
                &tmp_registry,
                Self::int_gauge("runtime_nodes_count", "Number of nodes in the runtime."),
            ),
            registry: tmp_registry,
        }
    }

    fn register<T: 'static + prometheus::core::Collector + Clone>(
        registry: &Registry,
        metric: T,
    ) -> T {
        registry.register(Box::new(metric.clone())).unwrap();
        metric
    }

    fn histogram(metric_name: &str, help: &str) -> Histogram {
        let opts = HistogramOpts::new(metric_name, help);
        Histogram::with_opts(opts).unwrap()
    }

    fn int_counter(metric_name: &str, help: &str) -> IntCounter {
        let opts = Opts::new(metric_name, help);
        IntCounter::with_opts(opts).unwrap()
    }

    fn int_gauge(metric_name: &str, help: &str) -> IntGauge {
        let opts = Opts::new(metric_name, help);
        IntGauge::with_opts(opts).unwrap()
    }

    pub fn gather(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics::new()
    }
}

trait MetricsCollector {
    fn inc_counter(metric_name: &str);
    fn start_histogram_timer(metric_name: &str) -> HistogramTimer;
    fn gather_metrics(&self) -> Vec<MetricFamily>;
}
