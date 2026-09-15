//! Human-lane ontology writes for the Brain board (#316).
//!
//! The board user *is* the human approver, so these routes apply entity and
//! link mutations directly — unlike the O3 proposal queue, where agents submit
//! drafts and only humans decide. Registered agent keys are refused here so an
//! agent cannot bypass its own propose-only lane.

use super::auth::MaybeHubPrincipal;
use super::hey::require_agent;
use super::{http_actor, json_error, AppState};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use axum::routing::post;
use axum::Router;
use serde::Deserialize;
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/ontology/entity", post(create_entity))
        .route("/api/ontology/link", post(create_link))
}

/// Refuse registered agent keys — agents propose, humans write.
async fn refuse_agents(
    state: &AppState,
    principal: &MaybeHubPrincipal,
    headers: &HeaderMap,
    request_id: &str,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if principal.agent_id().is_some() || require_agent(state, headers).await.is_ok() {
        return Err(json_error(
            StatusCode::FORBIDDEN,
            "agents propose; humans write",
            request_id,
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct EntityBody {
    /// `class` | `instance` | `metric`.
    kind: String,
    name: String,
    atom_id: Option<String>,
    attributes: Option<serde_json::Value>,
}

async fn create_entity(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    headers: HeaderMap,
    Json(body): Json<EntityBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_ontology_entity", request_id=%request_id);
    state.status.touch_client_activity();
    refuse_agents(&state, &principal, &headers, &request_id).await?;
    if body.name.trim().is_empty() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "name is required",
            &request_id,
        ));
    }
    let actor = http_actor(&principal);
    match crate::ontology::create_entity(
        state.brain.store().as_ref(),
        body.kind.trim(),
        body.name.trim(),
        body.atom_id.as_deref().map(str::trim),
        body.attributes,
    )
    .await
    {
        Ok(entity) => {
            #[cfg(feature = "postgres")]
            super::log_hub_write(
                &state,
                &principal,
                "ontology_entity",
                "http",
                Some(body.kind.as_str()),
                Some(&entity.id),
            )
            .await;
            Ok(Json(serde_json::json!({
                "ok": true,
                "request_id": request_id,
                "actor": actor,
                "entity": entity,
            })))
        }
        Err(e) => Err(json_error(
            StatusCode::BAD_REQUEST,
            e.to_string(),
            &request_id,
        )),
    }
}

#[derive(Debug, Deserialize)]
struct LinkBody {
    from_id: String,
    to_id: String,
    rel: String,
}

async fn create_link(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    headers: HeaderMap,
    Json(body): Json<LinkBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_ontology_link", request_id=%request_id);
    state.status.touch_client_activity();
    refuse_agents(&state, &principal, &headers, &request_id).await?;
    if body.from_id.trim().is_empty() || body.to_id.trim().is_empty() {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "from_id and to_id are required",
            &request_id,
        ));
    }
    let actor = http_actor(&principal);
    match crate::ontology::create_link(
        state.brain.store().as_ref(),
        body.from_id.trim(),
        body.to_id.trim(),
        body.rel.trim(),
        &actor,
    )
    .await
    {
        Ok(link) => {
            #[cfg(feature = "postgres")]
            super::log_hub_write(
                &state,
                &principal,
                "ontology_link",
                "http",
                Some(body.rel.as_str()),
                Some(&link.id),
            )
            .await;
            Ok(Json(serde_json::json!({
                "ok": true,
                "request_id": request_id,
                "link": link,
            })))
        }
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            Err(json_error(status, msg, &request_id))
        }
    }
}
