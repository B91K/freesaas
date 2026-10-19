use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use freesaas_core::{Error, IdentityProvider, StorageProvider, User};
use serde::Deserialize;

#[derive(Clone)]
pub struct AppState {
    pub identity: Arc<dyn IdentityProvider>,
    pub storage: Arc<BTreeMap<String, Arc<dyn StorageProvider>>>,
}

pub struct ApiError(Error);

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::InvalidPath(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Unsupported(_) => StatusCode::NOT_IMPLEMENTED,
            Error::Backend(_) => StatusCode::BAD_GATEWAY,
        };
        (
            status,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Deserialize)]
pub struct PathQuery {
    #[serde(default)]
    path: String,
}

async fn user(state: &AppState, headers: &HeaderMap) -> ApiResult<User> {
    let credential = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    Ok(state.identity.authenticate(credential).await?)
}

fn provider(state: &AppState, id: &str) -> ApiResult<Arc<dyn StorageProvider>> {
    state
        .storage
        .get(id)
        .cloned()
        .ok_or_else(|| Error::NotFound(format!("storage `{id}`")).into())
}

pub async fn me(State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Json<User>> {
    Ok(Json(user(&s, &headers).await?))
}

pub async fn storages(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<String>>> {
    user(&s, &headers).await?;
    Ok(Json(s.storage.keys().cloned().collect()))
}

pub async fn list(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<PathQuery>,
) -> ApiResult<impl IntoResponse> {
    let u = user(&s, &headers).await?;
    Ok(Json(provider(&s, &id)?.list(&u, &q.path).await?))
}

pub async fn read(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<PathQuery>,
) -> ApiResult<impl IntoResponse> {
    let u = user(&s, &headers).await?;
    let data = provider(&s, &id)?.read(&u, &q.path).await?;
    Ok(([(header::CONTENT_TYPE, "application/octet-stream")], data))
}

pub async fn write(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<PathQuery>,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let u = user(&s, &headers).await?;
    Ok(Json(provider(&s, &id)?.write(&u, &q.path, body).await?))
}

pub async fn delete(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<PathQuery>,
) -> ApiResult<StatusCode> {
    let u = user(&s, &headers).await?;
    provider(&s, &id)?.delete(&u, &q.path).await?;
    Ok(StatusCode::NO_CONTENT)
}
