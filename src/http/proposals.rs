//! O3 ontology proposal queue endpoints (#118).
//!
//! Agents submit drafts (`POST /api/ontology/proposals`, agent bearer); humans
//! decide (`POST /api/ontology/proposals/{id}/decide`). A pending proposal is
//! inert — approval is the only path that mutates entities/links, and agent
//! keys are refused on the decide route so agents can't approve their own.

use super::auth::MaybeHubPrincipal;
use super::hey::require_agent;
use super::{http_actor, json_error, AppState};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/ontology/proposals",
            get(list_proposals).post(submit_proposal),
        )
        .route("/api/ontology/proposals/{id}/decide", post(decide))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    status: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}

async fn list_proposals(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    state.status.touch_client_activity();
    let status = q.status.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(s) = status {
        if !matches!(
            s,
            crate::ontology::PROPOSAL_PENDING
                | crate::ontology::PROPOSAL_APPROVED
                | crate::ontology::PROPOSAL_REJECTED
        ) {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                "status must be pending|approved|rejected",
                &request_id,
            ));
        }
    }
    let proposals = state
        .brain
        .store()
        .list_ontology_proposals(status, q.limit.clamp(1, 500))
        .await
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "request_id": request_id,
        "proposals": proposals,
    })))
}

#[derive(Debug, Deserialize)]
struct ProposeBody {
    kind: String,
    #[serde(default)]
    payload: serde_json::Value,
    reason: Option<String>,
}

/// Agent-only draft submission. Bearer must resolve to a registered agent;
/// the proposal is `pending` and mutates nothing until a human decides.
async fn submit_proposal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProposeBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_ontology_propose", request_id=%request_id);
    state.status.touch_client_activity();
    let agent = require_agent(&state, &headers)
        .await
        .map_err(|s| json_error(s, "agent bearer required", &request_id))?;
    let proposer = agent.codename.clone();
    match crate::ontology::submit_proposal(
        state.brain.store().as_ref(),
        body.kind.trim(),
        body.payload,
        &proposer,
        body.reason,
    )
    .await
    {
        Ok(p) => Ok(Json(serde_json::json!({
            "ok": true,
            "request_id": request_id,
            "proposal": p,
        }))),
        Err(e) => Err(json_error(
            StatusCode::BAD_REQUEST,
            e.to_string(),
            &request_id,
        )),
    }
}

#[derive(Debug, Deserialize)]
struct DecideBody {
    /// `approve` | `reject`.
    action: String,
}

/// Human-only decision. An agent bearer is explicitly refused — agents
/// propose, humans approve.
async fn decide(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<DecideBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_ontology_decide", request_id=%request_id);
    state.status.touch_client_activity();

    let approve = match body.action.as_str() {
        "approve" => true,
        "reject" => false,
        other => {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                format!("action must be approve|reject, got {other}"),
                &request_id,
            ))
        }
    };

    // Registered agent keys are authorized callers in general, but deciding is
    // a human lane — refuse them here rather than letting agents self-approve.
    if principal.agent_id().is_some() || require_agent(&state, &headers).await.is_ok() {
        return Err(json_error(
            StatusCode::FORBIDDEN,
            "agents propose; humans decide",
            &request_id,
        ));
    }

    let actor = http_actor(&principal);
    match crate::ontology::decide_proposal(state.brain.store().as_ref(), id.trim(), approve, &actor)
        .await
    {
        Ok(p) => {
            #[cfg(feature = "postgres")]
            super::log_hub_write(
                &state,
                &principal,
                "ontology_decide",
                "http",
                Some(body.action.as_str()),
                Some(&id),
            )
            .await;
            Ok(Json(serde_json::json!({
                "ok": true,
                "request_id": request_id,
                "proposal": p,
            })))
        }
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else if msg.contains("already decided") {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            };
            Err(json_error(status, msg, &request_id))
        }
    }
}
