use std::sync::OnceLock;

use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider, trace::SdkTracerProvider};

use crate::profiler::Profiler;

/// Создает хранилище метрик и дает к нему доступ
pub fn get_resource(resource_name: String) -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| Resource::builder().with_service_name(resource_name).build())
        .clone()
}

/// Позволяет отправлять метрики перед окончанием работы сервиса
pub struct OtelGuard {
    // #[cfg(feature = "traces")]
    // pub tracer_provider: Option<SdkTracerProvider>,
    pub meter_provider: Option<SdkMeterProvider>,
}
impl Drop for OtelGuard {
    fn drop(&mut self) {
        // #[cfg(feature = "traces")]
        // if let Some(tracer) = &self.tracer_provider {
        //     unimplemented!();
        //     if let Err(err) = tracer.force_flush() {
        //         eprintln!("Failed to flush traces at process end {err:?}");
        //     };
        //
        //     if let Err(err) = tracer.shutdown() {
        //         eprintln!("Failed to close tracing provider successfully: {err:?}");
        //     }
        // }
        if let Some(metrics) = &self.meter_provider {
            if let Err(err) = metrics.force_flush() {
                eprintln!("Failed to flush traces at process end {err:?}");
            };
            if let Err(err) = metrics.shutdown() {
                eprintln!("Failed to close metrics provider successfully: {err:?}");
            }
        }
    }
}
