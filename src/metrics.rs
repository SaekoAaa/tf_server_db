use std::sync::OnceLock;
use std::time::Duration;

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::PeriodicReader;

use opentelemetry_sdk::metrics::SdkMeterProvider;

use crate::AppState;
use crate::otel::get_resource;

// Список метрик
pub(crate) static REQUESTS_COUNTER: OnceLock<Counter<u64>> = OnceLock::new();
pub(crate) static REQUESTS_LATENCY: OnceLock<Histogram<f64>> = OnceLock::new();

pub async fn http_metrics_middleware(req: Request, next: Next) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let method = req.method().clone();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|s| s.as_str().to_owned())
        .unwrap_or("".to_string());

    let response = next.run(req).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status();

    tracing::info!(
        "{} | path: {} method: {}, duration: {}",
        status,
        path,
        method,
        duration
    );
    let labels = [
        KeyValue::new("method", method.as_str().to_string()),
        KeyValue::new("path", path),
        KeyValue::new("status", status.as_u16() as i64),
    ];

    if let Some(counter) = REQUESTS_COUNTER.get() {
        counter.add(1, &labels);
    }

    if let Some(histogram) = REQUESTS_LATENCY.get() {
        histogram.record(duration, &labels);
    }
    response
}
/// Создает экспортер и сборщик метрик. Использует otlp формат
#[tracing::instrument]
pub fn init_metrics(collector_url: &str) -> anyhow::Result<SdkMeterProvider> {
    let export_duration = std::env::var("EXPORT_DURATION")
        .map(|e| e.parse::<u64>().unwrap_or(15))
        .unwrap_or(15);
    tracing::debug!("Telemetry address: {}", collector_url);
    let http_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_endpoint(collector_url)
        .build()?;
    let http_reader = PeriodicReader::builder(http_exporter)
        .with_interval(Duration::from_secs(export_duration))
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(http_reader)
        .with_resource(get_resource("todo_metrics".to_string()))
        .build();
    global::set_meter_provider(meter_provider.clone());
    let http_meter = global::meter("http");
    REQUESTS_COUNTER
        .set(
            http_meter
                .u64_counter("http_requests")
                .with_description("Shows amount of http requests")
                .build(),
        )
        .expect("Mertic already initialized");

    REQUESTS_LATENCY
        .set(
            http_meter
                .f64_histogram("requests_latency")
                .with_description("Shows time for request to take")
                .with_boundaries(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 10.0])
                .build(),
        )
        .expect("Mertic already initialized");
    tracing::debug!("Metrics initialized");
    Ok(meter_provider)
}
