use clap::{ArgAction, Parser, Subcommand};
use rmcp::transport::sse_server::MiddlewareFn;
use rmcp_proxy::{
    run_sse_client, run_sse_server,
    sse_client::SseClientConfig,
    sse_server::{SseServerSettings, StdioServerParameters},
};
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc, time::Duration};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod proxy;
use proxy::run_sse_proxy;
mod auth;
use auth::{ApiKeyVerifier, KeyVerifier, auth_middleware};
mod tsp;
use tsp::start_tsp_endpoint;

use crate::auth::TspKeyVerifier;

const AUTH_API_ADDR: &str = "http://127.0.0.1:8081";
const TSP_ENDPOINT: &str = "tcp://127.0.0.1:1337";

#[derive(Parser)]
#[command(
    name = "openmcp",
    version,
    about = "OpenMCP Server Proxy CLI",
    subcommand_required = true,
    arg_required_else_help = true,
    after_help = "Examples:\n\n \
        SSE > SSE : Proxy an SSE stream to a remote SSE server:\n \
        openmcp run -p exposed_host[default=localhost]:exposed_port:remote_sse_host:remote_sse_port\n\n \
        STDIO > SSE : Expose a local stdio server as an SSE server:\n \
        openmcp run -p exposed_host[default=localhost]:exposed_port\n \
        openmcp run -p 0.0.0.0:8080 -- npx -y @modelcontextprotocol/server-everythingt\n \
        openmcp run -p :8080 -- python mcp_server.py\n\n \
        SSE > STDIO : Connect to a remote server over SSE and expose it as a stdio server:\n \
        openmcp run -p remote_sse_host:remote_sse_port\n\n \
        STDIO > STDIO : This case is currently unused and not implemented.\n \
        "
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the proxy
    Run {
        #[arg(short = 'H', long = "headers", value_name = "KEY=VALUE", value_parser = pair_parser)]
        headers: Vec<(String, String)>,

        /// Any extra arguments to the command to spawn the server
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Environment variables used when spawning the server. Can be used multiple times.
        #[arg(short = 'e', long = "env", value_name = "KEY=VALUE", value_parser = pair_parser)]
        env_vars: Vec<(String, String)>,

        /// Forward remote SSE server or local stdio server as an SSE server.
        #[arg(short = 'p')]
        publish: Option<String>,

        // Enable security for the server, such as token authentication.
        #[arg(long = "security", action = ArgAction::SetTrue)]
        security: bool,

        /// Extend TSP protocol to [verify | message] mode.
        #[arg(long = "tsp", value_name = "MODE", action = ArgAction::Append, value_delimiter = ',', value_parser = ["verify", "message"], conflicts_with = "security")]
        tsp_modes: Vec<String>,
    },
}

fn pair_parser(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        Err(format!("Invalid env format: {}", s))
    } else {
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

fn parse_publish(input: &str) -> Vec<String> {
    let placeholder = "__DELIM__";
    let input_with_placeholder = input.replace("://", placeholder);
    let parts = input_with_placeholder.split(':').collect::<Vec<&str>>();
    let result: Vec<String> = parts
        .into_iter()
        .map(|part| part.replace(placeholder, "://"))
        .collect();
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let cli = Cli::parse();
    let mut middlewares: Vec<MiddlewareFn> = Vec::new();

    match cli.command {
        Commands::Run {
            headers,
            mut args,
            env_vars,
            publish,
            security,
            tsp_modes,
        } => {
            if security {
                let verifier = Arc::new(ApiKeyVerifier::new(AUTH_API_ADDR.to_string()))
                    as Arc<dyn KeyVerifier>;
                middlewares.push(auth_middleware(verifier));
            }

            for modes in &tsp_modes {
                if modes == "message" {
                    debug!("TSP mode: message is not supported yet, skipping");
                } else {
                    let (store, vid) = start_tsp_endpoint(TSP_ENDPOINT).await?;
                    let store = store.read().await.clone();
                    let verifier =
                        Arc::new(TspKeyVerifier::new(store, vid)) as Arc<dyn KeyVerifier>;
                    middlewares.push(auth_middleware(verifier));
                }
            }

            if args.len() > 0 {
                if let Some(publish_str) = publish {
                    let command = args.remove(0);
                    let mapping = parse_publish(&publish_str);
                    let bind_addr = match mapping.as_slice() {
                        [ip, port] => format!("{}:{}", ip, port),
                        [port] => format!("127.0.0.1:{}", port),
                        _ => {
                            error!(
                                "Invalid publish format with command. Expected <exposed_sse_ip>[option]:<exposed_sse_port>"
                            );
                            std::process::exit(1);
                        }
                    };
                    let mut env_map: HashMap<String, String> = HashMap::new();
                    for (key, value) in &env_vars {
                        env_map.insert(key.clone(), value.clone());
                    }

                    let stdio_params = StdioServerParameters {
                        command: command,
                        args: args,
                        env: env_map,
                    };

                    let sse_settings = SseServerSettings {
                        bind_addr: bind_addr.parse::<SocketAddr>()?,
                        keep_alive: Some(Duration::from_secs(15)),
                        middlewares: Some(Arc::new(middlewares)),
                    };
                    info!(
                        "SSE server listening on {} to local stdio server",
                        sse_settings.bind_addr
                    );
                    run_sse_server(stdio_params, sse_settings).await?;
                } else {
                    error!(
                        "Run stdio server without publish parameter. Expected <exposed_sse_ip>[option]:<exposed_sse_port>"
                    );
                    std::process::exit(1);
                }
            } else {
                if let Some(publish_str) = publish {
                    let mapping = parse_publish(&publish_str);
                    let (bind_addr, remote_addr) = match mapping.as_slice() {
                        [ip, port, remote_sse_ip, remote_sse_port] => (
                            Some(format!("{}:{}", ip, port)),
                            format!("{}:{}", remote_sse_ip, remote_sse_port),
                        ),
                        [port, remote_sse_ip, remote_sse_port] => (
                            Some(format!("127.0.0.1:{}", port)),
                            format!("{}:{}", remote_sse_ip, remote_sse_port),
                        ),
                        [ip, port] => (None, format!("{}:{}", ip, port)),
                        _ => {
                            error!(
                                "Invalid publish format with command. Expected <exposed_sse_ip>[option]:<exposed_sse_port>:<remote_sse_ip>:<remote_sse_port>"
                            );
                            std::process::exit(1);
                        }
                    };

                    let mut headers_map: HashMap<String, String> = HashMap::new();
                    for (key, value) in &headers {
                        headers_map.insert(key.clone(), value.clone());
                    }

                    let remote_config = SseClientConfig {
                        url: remote_addr.clone(),
                        headers: headers_map,
                    };

                    if let Some(bind_addr) = bind_addr {
                        let exposed_settings = SseServerSettings {
                            bind_addr: bind_addr.parse::<SocketAddr>()?,
                            keep_alive: Some(Duration::from_secs(15)),
                            middlewares: Some(Arc::new(middlewares)),
                        };
                        info!(
                            "SSE server listening on {}, proxying to {}",
                            exposed_settings.bind_addr, remote_addr
                        );
                        run_sse_proxy(remote_config, exposed_settings).await?;
                    } else {
                        info!(
                            "Connecting to remote SSE server at {} as stdio MCP client",
                            remote_addr
                        );
                        run_sse_client(remote_config).await?;
                    }
                } else {
                    error!("Missing both publish parameter and command.");
                    std::process::exit(1);
                }
            }
        }
    }
    Ok(())
}
