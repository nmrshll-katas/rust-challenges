use crate::actor_party_matcher::RequestMsg;
use crate::ServerState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::time::Duration;
use tokio::sync::oneshot;

pub async fn wait_for_other_party(
    Path(id): axum::extract::Path<String>,
    State(state): axum::extract::State<ServerState>,
) -> Result<Response<Body>, HandlerErr> {
    let (tx_resp, rx_resp) = oneshot::channel::<()>();

    state
        .sender_party_matcher
        .send(RequestMsg::CheckIn {
            id: id.clone(),
            tx_resp,
        })
        .await?;

    // race between rx_resp and timeout
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            // check-out: help the state actor garbage-collect:
            // if I know I'm about to timeout, send a check-out so the actor can delete the map entry
            state.sender_party_matcher.send(RequestMsg::CheckOut { id }).await?;
            return Err(HandlerErr::Timeout);
        }
        // rx_resp implements future so can be awaited directly without .recv()
        resp = rx_resp => {
            println!("received resp from actor: {:?}", resp);
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .body("event: ping\ndata: {\"message\":\"Other party is ready\"}\n\n".into())
        .map_err(|e| HandlerErr::Http(e))
}

#[derive(thiserror::Error, Debug)]
pub enum HandlerErr {
    #[error("HandlerErr: {0}")]
    Http(axum::http::Error),
    #[error("Failed sending Request to state actor: {0}")]
    Sender(#[from] tokio::sync::mpsc::error::SendError<RequestMsg>),
    #[error("Timeout: request elapsed 10s")]
    Timeout,
}
impl IntoResponse for HandlerErr {
    fn into_response(self) -> axum::response::Response {
        match &self {
            HandlerErr::Http(_) | HandlerErr::Sender(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL SERVER ERROR".to_string(),
            ),
            HandlerErr::Timeout => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
        }
        .into_response()
    }
}
