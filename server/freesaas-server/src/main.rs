mod api;
mod providers;

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use freesaas_core::config::Config;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

fn load_config() -> Result<Config, String> {
    let path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("FREESAAS_CONFIG").ok())
        .unwrap_or_else(|| "freesaas.yaml".into());
    let text = std::fs::read_to_string(&path).map_err(|e| format!("reading {path}: {e}"))?;
    serde_yaml::from_str(&text).map_err(|e| format!("parsing {path}: {e}"))
}

fn build_state(cfg: &Config) -> Result<api::AppState, String> {
    let identity = providers::identity(&cfg.identity)?;
    let mut storage = BTreeMap::new();
    for s in &cfg.storage {
        let p = providers::storage(s)?;
        if storage.insert(p.id().to_string(), p).is_some() {
            return Err(format!(
                "two storage providers share the id `{}`; give one an `id`",
                s.id_or_kind()
            ));
        }
    }
    Ok(api::AppState {
        identity,
        storage: Arc::new(storage),
    })
}

pub fn router(state: api::AppState, web_root: &str) -> Router {
    let api = Router::new()
        .route("/me", get(api::me))
        .route("/storage", get(api::storages))
        .route("/storage/{id}/list", get(api::list))
        .route(
            "/storage/{id}/file",
            get(api::read).put(api::write).delete(api::delete),
        );
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .nest("/api", api)
        .fallback_service(ServeDir::new(web_root))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let result = async {
        let cfg = load_config()?;
        let state = build_state(&cfg)?;
        let web_root = std::env::var("FREESAAS_WEB_ROOT").unwrap_or_else(|_| "web".into());
        let listener = tokio::net::TcpListener::bind(&cfg.listen)
            .await
            .map_err(|e| format!("binding {}: {e}", cfg.listen))?;
        tracing::info!("FreeSaaS listening on http://{}", cfg.listen);
        axum::serve(listener, router(state, &web_root))
            .await
            .map_err(|e| e.to_string())
    }
    .await;

    if let Err(e) = result {
        eprintln!("freesaas: {e}");
        std::process::exit(1);
    }
}
