// SPDX-License-Identifier: GPL-3.0-or-later
use evdev::KeyCode;
use prometheus::{
    GaugeVec, Histogram, IntCounterVec, Registry, histogram_opts, labels, opts, push_metrics,
};
use tokio::task::{JoinHandle, spawn_blocking};
use tracing::error;

pub struct MetricsManager {
    registry: Registry,
    keypress_counter: IntCounterVec,
    key_latency_histogram: Histogram,
    wpm_gauge: GaugeVec,
}

impl MetricsManager {
    pub fn new() -> prometheus::Result<Self> {
        let registry = Registry::new();

        let keypress_counter = IntCounterVec::new(
            opts!("key_presses_total", "Total number of key presses"),
            &["user", "keyboard", "key", "layout"],
        )?;

        let key_latency_histogram = Histogram::with_opts(histogram_opts!(
            "key_latency_secs",
            "Latency between key press and report",
            vec![0.00001, 0.0001, 0.001, 0.01, 0.025, 0.05, 0.1, 0.25]
        ))?;

        let wpm_gauge = GaugeVec::new(
            opts!("wpm", "Words per minute"),
            &["user", "keyboard", "layout"],
        )?;

        registry.register(Box::new(keypress_counter.clone()))?;
        registry.register(Box::new(key_latency_histogram.clone()))?;
        registry.register(Box::new(wpm_gauge.clone()))?;

        Ok(Self {
            registry,
            keypress_counter,
            key_latency_histogram,
            wpm_gauge,
        })
    }

    pub fn register_key_event(&self, key: &KeyCode, keyboard: &str) {
        self.keypress_counter
            .with_label_values(&[
                "ytropek".into(),
                keyboard.into(),
                self.key_name(key),
                "qwerty".into(),
            ])
            .inc();
    }

    pub fn register_key_to_report_time(&self, time: u64) {
        self.key_latency_histogram
            .observe((time as f64) / 1_000_000_000.0);
    }

    pub fn register_wpm(&self, wpm: u16) {
        self.wpm_gauge
            .with_label_values(&["ytropek", "KeychronQ10", "qwerty"])
            .set(wpm.into());
    }

    // pub async fn start_server(&self) {
    // use warp::{Filter, http::StatusCode};
    // let metrics_route = warp::path("metrics").map(|| {
    //     let mut buffer = Vec::new();
    //     let encoder = TextEncoder::new();
    //     let metrics = prometheus::gather();
    //     encoder.encode(&metrics, &mut buffer).unwrap();
    //     warp::http::Response::builder()
    //         .header("Content-Type", encoder.format_type())
    //         .body(buffer)
    // });
    //
    // warp::serve(metrics_route).run(([127, 0, 0, 1], 9095)).await;
    // }
    //
    // pub async fn stop_server(&self) {
    // if let Some(tx) = self.shutdown_trigger.lock().unwrap().take() {
    //     let _ = tx.send(()); // Signal shutdown
    // }
    // }

    pub async fn push(&self) -> JoinHandle<()> {
        let reg = self.registry.gather();

        spawn_blocking(move || {
            if let Err(err) = push_metrics("charon", labels! {}, "http://localhost:9091", reg, None)
            {
                error!("Error while pushing metrics: {err}");
            }
        })
    }

    fn key_name(&self, key: &KeyCode) -> String {
        let txt = format!("{key:?}");
        txt.replace("KEY_", "")
    }
}
