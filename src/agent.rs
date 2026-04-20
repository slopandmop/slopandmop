// sm-agent — the only sm-side service on the public tunnel. axum on :8080.
// Routes:
//   GET  /health    — boot status, deployments the sm has provisioned
//   GET  /chat*     — cookie-verified reverse proxy to 127.0.0.1:7682 (ttyd)
//   *               — 404
//
// The cookie is HS256-signed by the factory; key is in SM_COOKIE_KEY env.
// First request may come as `?session=<cookie>` — we set it as an httpOnly
// Secure cookie and 303 to /chat/.

use anyhow::Result;
use axum::{
    routing::{any, get},
    Router,
};

pub async fn run() -> Result<()> {
    let cfg = crate::config::SmEnv::from_env()?;
    tracing::info!(sm_id = %cfg.sm_id, "sm-agent starting");

    let app = Router::new()
        .route("/health", get(todo_endpoint))
        .route("/chat", any(todo_endpoint))
        .route("/chat/*rest", any(todo_endpoint));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("sm-agent listening on :8080");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn todo_endpoint() -> (axum::http::StatusCode, &'static str) {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        "endpoint not yet implemented — see agent.rs TODO",
    )
}
