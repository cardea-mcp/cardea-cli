# TSP Extend

## Verify mode

To run OpenMCP with TSP verification mode, use the following command:

```bash
RUST_LOG=openmcp=info cargo run --bin openmcp run -p 8000 --tsp=verify npx -y @modelcontextprotocol/server-everything
```

You should see a log message like:

```log
INFO openmcp::tsp: TSP endpoint listening for incoming messages to: did:peer:2.Vz6Mv1jmUsrF9KMgYcEmPSHddRUruXrWWhpynxFxEmFEud6Z8.Ez6Lc6MTmvySELeduDHdX4qb95KSHAPGV2duC8zksqUcBnhQn.SeyJzIjp7InVyaSI6InRjcDovLzEyNy4wLjAuMToxMzM3In0sInQiOiJ0c3AifQ
```

Then open another terminal session and run the following command with the endpoint VID copied from above:

```bash
cargo run --bin tsp_token_generator did:peer:2.Vz6Mv1jmUsrF9KMgYcEmPSHddRUruXrWWhpynxFxEmFEud6Z8.Ez6Lc6MTmvySELeduDHdX4qb95KSHAPGV2duC8zksqUcBnhQn.SeyJzIjp7InVyaSI6InRjcDovLzEyNy4wLjAuMToxMzM3In0sInQiOiJ0c3AifQ
```

This will output a Bearer token:
> We currently use the client VID directly as the Bearer token to identify the MCP client. Paste this token into your inspector tool to test how it works.

```log
Bearer Token: did:peer:2.Vz6Mv3NzkMtA4VfDb3UZncJucxW4tvKhBNwmhJMdjiy2Gs1GT.Ez6LbyhFYMNHac2SnaiVKFSKryiPi63dXParfVHpwGgdhFCV7.SeyJzIjp7InVyaSI6InRjcDovLzEyNy4wLjAuMToxMzM4In0sInQiOiJ0c3AifQ
```
