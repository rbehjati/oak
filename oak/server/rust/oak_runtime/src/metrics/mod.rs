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

use log::warn;
use prometheus::{
    proto::MetricFamily, Histogram, HistogramOpts, HistogramTimer, IntCounter, IntGauge, Opts,
    Registry,
};
use std::{collections::HashMap, sync::RwLock};

pub mod server;

/// Enum to allow retrieval of metrics of different types from the `metrics_info` HashMap
#[derive(Clone)]
enum MetricItem {
    Counter(IntCounter),
    Gauge(IntGauge),
    Hist(Histogram),
}

/// Struct that collects all the metrics in one place
pub struct Metrics {
    registry: Registry,

    /// Collection of all metrics, for easy lookup using a metric's name
    metrics_info: RwLock<HashMap<String, MetricItem>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            metrics_info: RwLock::new(HashMap::new()),
        }
    }

    fn register<T: 'static + prometheus::core::Collector + Clone>(
        &self,
        metric: T,
        metric_item: MetricItem,
        metric_name: &str,
    ) {
        self.registry.register(Box::new(metric)).unwrap();
        self.metrics_info
            .write()
            .expect("could not acquire lock on metrics_info")
            .insert(metric_name.to_string(), metric_item);
    }

    fn get_metric(&self, metric_name: &str) -> Option<MetricItem> {
        self.metrics_info
            .read()
            .expect("Could not acquire the lock for metrics_info.")
            .get(metric_name)
            .cloned()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics::new()
    }
}

pub trait MetricsCollector {
    fn register_histogram(&self, metric_name: &str, help: &str);
    fn register_int_counter(&self, metric_name: &str, help: &str);
    fn register_int_gauge(&self, metric_name: &str, help: &str);

    fn inc_int_counter(&self, metric_name: &str);
    fn set_int_gauge(&self, metric_name: &str, val: i64);
    fn add_to_histogram(&self, metric_name: &str, val: f64);
    fn start_histogram_timer(&self, metric_name: &str) -> Option<HistogramTimer>;

    fn gather_metrics(&self) -> Vec<MetricFamily>;
}

impl MetricsCollector for Metrics {
    fn register_histogram(&self, metric_name: &str, help: &str) {
        let opts = HistogramOpts::new(metric_name, help);
        let hist = Histogram::with_opts(opts).unwrap();
        self.register(hist.clone(), MetricItem::Hist(hist), metric_name);
    }

    fn register_int_counter(&self, metric_name: &str, help: &str) {
        let opts = Opts::new(metric_name, help);
        let counter = IntCounter::with_opts(opts).unwrap();
        self.register(counter.clone(), MetricItem::Counter(counter), metric_name);
    }

    fn register_int_gauge(&self, metric_name: &str, help: &str) {
        let opts = Opts::new(metric_name, help);
        let gauge = IntGauge::with_opts(opts).unwrap();
        self.register(gauge.clone(), MetricItem::Gauge(gauge), metric_name);
    }

    fn inc_int_counter(&self, metric_name: &str) {
        if let Some(metric) = self.get_metric(metric_name) {
            match metric {
                MetricItem::Counter(cnt) => cnt.inc(),
                _ => warn!("{} is not an IntCounter.", metric_name),
            }
        }
    }

    fn set_int_gauge(&self, metric_name: &str, val: i64) {
        if let Some(metric) = self.get_metric(metric_name) {
            match metric {
                MetricItem::Gauge(gauge) => gauge.set(val),
                _ => warn!("{} is not an IntGauge.", metric_name),
            }
        }
    }

    fn add_to_histogram(&self, metric_name: &str, val: f64) {
        if let Some(metric) = self.get_metric(metric_name) {
            match metric {
                MetricItem::Hist(hist) => hist.observe(val),
                _ => warn!("{} is not a Histogram.", metric_name),
            }
        }
    }

    fn start_histogram_timer(&self, metric_name: &str) -> Option<HistogramTimer> {
        self.get_metric(metric_name).and_then(|m| match m {
            MetricItem::Hist(hist) => Some(hist.start_timer()),
            _ => {
                warn!("{} is not a Histogram.", metric_name);
                None
            }
        })
    }

    fn gather_metrics(&self) -> Vec<MetricFamily> {
        self.registry.gather()
    }
}
