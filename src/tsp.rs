use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{error, info};
use tsp_sdk::{AsyncSecureStore, VerifiedVid, definitions::ReceivedTspMessage, vid::OwnedVid};

pub async fn start_tsp_endpoint(endpoint: &str) -> Result<(Arc<RwLock<AsyncSecureStore>>, String)> {
    let store = Arc::new(RwLock::new(AsyncSecureStore::new()));
    let url: url::Url = endpoint.parse()?;
    let vid = OwnedVid::new_did_peer(url);
    let vid_str = vid.identifier().to_string();
    {
        let guard = store.write().await;
        guard.add_private_vid(vid.clone(), None)?;
    }
    info!(
        "TSP endpoint listening for incoming messages to: {}",
        vid.identifier()
    );
    tokio::spawn({
        let store = Arc::clone(&store);
        let vid_str_clone = vid_str.clone();
        async move {
            listener(store, &vid_str_clone).await.unwrap_or_else(|e| {
                error!("Error in listener: {e}");
            });
        }
    });
    Ok((store, vid_str))
}

async fn listener(store: Arc<RwLock<AsyncSecureStore>>, host_vid: &str) -> Result<()> {
    let mut stream = {
        let mut guard = store.write().await;
        guard.receive(host_vid).await?
    };
    while let Some(result) = stream.next().await {
        match result {
            Ok(ReceivedTspMessage::RequestRelationship {
                sender, thread_id, ..
            }) => {
                info!("🤝 Received relationship request from: {sender}");
                let guard = store.write().await;
                match guard
                    .send_relationship_accept(host_vid, &sender, thread_id, None)
                    .await
                {
                    Ok(_) => info!("✅ Sent relationship_accept to {sender}"),
                    Err(err) => error!("Failed to send relationship_accept to {sender}: {err}"),
                }
            }
            Ok(msg) => {
                info!("Received message: {:?}", msg);
            }
            Err(e) => {
                error!("Receive error: {e:?}");
            }
        }
    }
    Ok(())
}
