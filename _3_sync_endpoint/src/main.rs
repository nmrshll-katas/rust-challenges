use actor_party_matcher::{ActorPartyMatcher, RequestMsg};
use axum::{routing::get, Router};
use handlers::wait_for_other_party;
use reqwest::RequestBuilder;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

// to match callers by pair (by unique_id), we'll need a mapping of unique_id => sender
// we want concurrent access to the mapping of unique_id => sender,
// so we'll create an actor: a tokio task that owns the mapping of unique_id => sender
pub mod actor_party_matcher;
pub mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::spawn().await;
    tokio::try_join!(server.join_handle_server, server.join_handle_actor_matcher)?;
    Ok(())
}

#[derive(Clone)]
pub struct ServerState {
    pub sender_party_matcher: Sender<RequestMsg>,
}

pub struct Server {
    join_handle_server: JoinHandle<()>,
    join_handle_actor_matcher: JoinHandle<()>,
    port: u16,
}
impl Server {
    pub fn router() -> Router<ServerState> {
        Router::new()
            .route("/health", get(|| async { "ok" }))
            .route(
                "/wait-for-second-party/:unique-id",
                get(wait_for_other_party),
            )
    }
    pub async fn spawn() -> Self {
        let actor_party_matcher = ActorPartyMatcher::spawn();
        let state = ServerState {
            sender_party_matcher: actor_party_matcher.sender,
        };

        let router = Self::router().with_state(state);
        // Bind to localhost at the port 0, which will let the OS assign an available port to us
        let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
        // Retrieve the port assigned to us by the OS
        let port = listener.local_addr().unwrap().port();

        let join_handle_server = tokio::spawn(async move {
            println!("Listening on http://0.0.0.0:{}", port);
            axum::serve(listener, router).await.unwrap();
        });

        Self {
            join_handle_server,
            join_handle_actor_matcher: actor_party_matcher.join_handle,
            port,
        }
    }
    pub fn base_url(&self) -> String {
        format!("http://0.0.0.0:{}", self.port)
    }
    pub fn url(&self, url_path: &str) -> String {
        if url_path.starts_with("http") {
            return url_path.to_string();
        }
        let path = url_path.trim().trim_start_matches('/');
        format!("http://0.0.0.0:{}/{}", self.port, path)
    }
    pub fn get(&self, url_path: &str) -> RequestBuilder {
        reqwest::Client::new().get(self.url(url_path))
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use reqwest::StatusCode;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_health_ok() {
        let server = Server::spawn().await;
        let resp = server.get("/health").send().await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test(start_paused = true)]
    async fn wait_for_sse_err__timeout__no_other_party() {
        let server = Server::spawn().await;
        let resp = server
            .get("/wait-for-second-party/123")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[tokio::test(start_paused = true)]
    async fn wait_for_sse_ok__found_other_party() -> Result<(), Box<dyn std::error::Error>> {
        let server = Server::spawn().await;

        let fut_req1 = server.get("/wait-for-second-party/123").send();
        let fut_req2 = server.get("/wait-for-second-party/123").send();

        let (resp1, resp2) = tokio::join!(fut_req1, fut_req2);

        assert_eq!(resp1?.status(), StatusCode::OK);
        assert_eq!(resp2?.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test(start_paused = true)]
    async fn wait_for_sse_err__timeout__other_parties_but_no_match(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let server = Server::spawn().await;

        let mut join_set = JoinSet::new();
        for i in 1..10 {
            let fut_req = server.get(&format!("/wait-for-second-party/{i}")).send();
            join_set.spawn(fut_req);
        }
        let results = join_set.join_all().await;

        for result in results {
            assert_eq!(result?.status(), StatusCode::REQUEST_TIMEOUT);
        }

        Ok(())
    }
}
