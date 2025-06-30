use futures::StreamExt;
use std::env;
use tsp_sdk::{AsyncSecureStore, OwnedVid, VerifiedVid, definitions::ReceivedTspMessage};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <peer_endpoint_uri>", args[0]);
        std::process::exit(1);
    }
    let peer = &args[1];

    let mut store = AsyncSecureStore::new();
    let vid = OwnedVid::new_did_peer("tcp://127.0.0.1:1338".parse().unwrap());
    store.add_private_vid(vid.clone(), None).unwrap();
    store
        .verify_vid(peer, None)
        .await
        .expect("failed to verify peer VID");
    store
        .send_relationship_request(vid.identifier(), &peer, None)
        .await
        .expect("failed to send relationship request");
    let mut messages = store
        .receive(vid.identifier())
        .await
        .expect("failed to start receive");
    let ReceivedTspMessage::AcceptRelationship { .. } =
        messages.next().await.unwrap().unwrap()
    else {
        panic!("did not receive a relation accept");
    };
    println!("Bearer Token: {}", vid.identifier());
}
