// Factory — the axum service on the factory TDX VM. Only thing that has
// compute.admin on our GCP project. See README.md §factory and the plan for
// the endpoint contracts.
//
// Endpoints (all under https://factory.slopandmop.com):
//   POST /sm/provision           public + CAPTCHA + IP rate limit → {chat_url, ttl_seconds}
//   POST /sm/register            ITA-gated → {tunnel_token, hostnames}
//   POST /sm/provision-workload  ITA-gated + registry lookup → {child_id, public_url}
//   POST /sm/teardown-workload   ITA-gated + registry lookup → 204
//   GET  /sm/{id}/health         registry-auth'd proxy to child agent /health
//
// Persistence: JSON file at $REGISTRY_PATH, atomic-written on every mutation,
// rehydrated on boot. See watchdog.rs for teardown reaper.

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};

pub async fn run() -> Result<()> {
    let cfg = crate::config::FactoryEnv::from_env()?;
    tracing::info!(?cfg.factory_url, ?cfg.gcp_project, "factory starting");

    // TODO: load registry from disk, spawn watchdog::run(registry.clone()).
    let _ = cfg;

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/sm/provision", post(todo_endpoint))
        .route("/sm/register", post(todo_endpoint))
        .route("/sm/provision-workload", post(todo_endpoint))
        .route("/sm/teardown-workload", post(todo_endpoint))
        .route("/sm/:id/health", get(todo_endpoint));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("factory listening on :8080");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn todo_endpoint() -> (axum::http::StatusCode, &'static str) {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        "endpoint not yet implemented — see factory.rs TODO",
    )
}
