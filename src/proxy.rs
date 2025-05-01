use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo},
    transport::{
        sse::SseTransport,
        sse_server::{SseServer, SseServerConfig},
    },
};
use rmcp_proxy::{
    proxy_handler::ProxyHandler, sse_client::SseClientConfig, sse_server::SseServerSettings,
};
use std::error::Error as StdError;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run_sse_proxy(
    config: SseClientConfig,
    sse_settings: SseServerSettings,
) -> Result<(), Box<dyn StdError>> {
    info!("Running SSE proxy with URL: {}", config.url);

    // Create SSE transport with default client
    // Note: We're not using custom headers right now, but we could extend this in the future
    if !config.headers.is_empty() {
        info!("Note: Custom headers are not currently supported for SSE transport");
    }

    // Create SSE transport
    let transport = SseTransport::start(&config.url).await?;

    // Create client info with full capabilities to ensure we can use all the server's features
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::builder()
            .enable_experimental()
            .enable_roots()
            .enable_roots_list_changed()
            .enable_sampling()
            .build(),
        ..Default::default()
    };

    // Create client service with transport
    let client = client_info.serve(transport).await?;

    // Get server info
    let server_info = client.peer_info();
    info!("Connected to server: {}", server_info.server_info.name);

    // Create proxy handler
    let proxy_handler = ProxyHandler::new(client);

    // Configure SSE server
    let config = SseServerConfig {
        bind: sse_settings.bind_addr,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: CancellationToken::new(),
        middlewares: sse_settings.middlewares,
    };

    // Start the SSE server
    let sse_server = SseServer::serve_with_config(config.clone()).await?;

    // Register the proxy handler with the SSE server
    let ct = sse_server.with_service(move || proxy_handler.clone());

    // Wait for Ctrl+C to shut down
    tokio::signal::ctrl_c().await?;
    ct.cancel();

    Ok(())
}
