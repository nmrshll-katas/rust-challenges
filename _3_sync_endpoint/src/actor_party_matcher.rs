use std::collections::BTreeMap;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// The message sent by the request handler to this actor
pub enum RequestMsg {
    /// Notify the actor that there is a caller waiting with this ID
    CheckIn {
        id: String,
        // why oneshot when have 2 parties to notify ? we already have another oneshot stored for the first party to sign-in
        tx_resp: oneshot::Sender<()>,
    },
    /// Notify the actor that the caller with this ID has timed out and their request is cancelled
    CheckOut { id: String },
}
/// The reponse sent by this actor to the request handler
pub enum RespMsg {
    NoOtherPartyWaiting,
    OtherPartyReady,
}

pub struct ActorPartyMatcher {
    pub join_handle: JoinHandle<()>,
    pub sender: mpsc::Sender<RequestMsg>,
}
impl ActorPartyMatcher {
    pub fn spawn() -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<RequestMsg>(16);
        // Mutable state, fully owned by this actor.  No other task/thread is trying to access it concurrently, so we can mutate it sequentially.
        // TODO (with more time) we could also implement automated garbage-collection (based on time of insert, which we could store). That would make it impossible for this map to grow forever bigger.
        let mut parties_waiting: BTreeMap<String, oneshot::Sender<()>> = BTreeMap::new();

        let join_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    RequestMsg::CheckIn {
                        id,
                        tx_resp: tx_resp_incoming_party,
                    } => {
                        println!("received check-in with ID: {id}");
                        if let Some(chan_resp_waiting_party) = parties_waiting.remove(&id) {
                            println!("found other party waiting with ID: {id}");
                            // another party is already waiting with the same ID
                            // notify the other party's request handler that both parties are ready
                            match chan_resp_waiting_party.send(()) {
                                Ok(_) => {}
                                Err(err) => {
                                    eprintln!("Failed sending response to waiting party: {err:?}");
                                }
                            }
                            match tx_resp_incoming_party.send(()) {
                                Ok(_) => {}
                                Err(err) => {
                                    eprintln!("Failed sending response to incoming party: {err:?}");
                                }
                            }
                        } else {
                            println!("found NO other party with ID: {id}. Will wait...");
                            // no other party waiting. store the channel for later use
                            parties_waiting.insert(id, tx_resp_incoming_party);
                        }
                    }
                    RequestMsg::CheckOut { id } => {
                        println!("received check-out with ID: {id}");
                        parties_waiting.remove(&id);
                    }
                }
            }
        });

        Self {
            join_handle,
            sender: tx,
        }
    }
}
