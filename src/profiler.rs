use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use pyroscope::{PyroscopeAgent, pyroscope::PyroscopeAgentRunning};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};

use crate::{AppState, profiler};

pub struct Profiler {
    agent: PyroscopeAgent<PyroscopeAgentRunning>,
}
impl Profiler {
    pub fn new(pyroscope_url: &str, app_name: &str) -> pyroscope::Result<Self> {
        let agent = PyroscopeAgent::builder(pyroscope_url, app_name)
            .backend(pprof_backend(PprofConfig::new().sample_rate(1000)))
            .build()?;
        let agent_running = agent.start()?;
        Ok(Self {
            agent: agent_running,
        })
    }
    pub fn stop(self) -> pyroscope::Result<()> {
        let agent_ready = self.agent.stop()?;
        agent_ready.shutdown();
        Ok(())
    }
    pub async fn wrap<F, Fut, T>(&self, key: &str, value: &str, f: F) -> pyroscope::Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let (add_tag, remove_tag) = self.agent.tag_wrapper();

        add_tag(key.to_string(), value.to_string())?;
        tracing::info!("Added profile: {key} - {value}");
        let result = f().await;

        let _ = remove_tag(key.to_string(), value.to_string());

        Ok(result)
    }
}
pub async fn profiling_middleware(
    State(profiler): State<Arc<Profiler>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    tracing::info!("Profiled!");
    profiler
        .wrap("route", &path, || async { next.run(request).await })
        .await
        .unwrap()
}
pub async fn maybe_profile<F, Fut, T>(
    profiler: Option<Arc<Profiler>>,
    key: &str,
    value: &str,
    f: F,
) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    if let Some(profiler) = profiler {
        profiler.wrap(key, value, f).await.unwrap()
    } else {
        f().await
    }
}
