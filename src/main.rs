use std::sync::Arc;

use libp2p::{futures::StreamExt, identify, relay::Event, swarm::SwarmEvent, Multiaddr};
use tracing::{error, info};

use crate::node::NodeBehaviourEvent;
use crate::metrics::MetricsCollector;
use crate::webserver::create_router;

mod node;
mod metrics;
mod webserver;

fn get_listen_addrs() -> Vec<String> {

    vec![
        format!("/ip4/0.0.0.0/tcp/3000/ws"),
        format!("/ip6/::/tcp/3000/ws"),
        // format!("/ip4/0.0.0.0/tcp/3000"),
        // format!("/ip6/::/tcp/3000"),
    ]
}

#[tokio::main]
async fn main() {
    // Initialize tracing with more detailed format
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting proxy server...");
    let mut swarm = node::create_swarm().await.expect("Failed to create swarm");
    
    // Initialize metrics collector with peer ID
    let peer_id = swarm.local_peer_id().to_string();
    let metrics = Arc::new(MetricsCollector::new(peer_id));
    
    // Start web server in background
    let web_metrics = metrics.clone();
    tokio::spawn(async move {
        let app = create_router(web_metrics);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
        info!("Web server listening on http://0.0.0.0:80");
        axum::serve(listener, app).await.unwrap();
    });

    for addr in get_listen_addrs() {
        let addr : Multiaddr = addr.parse().unwrap();
        swarm.listen_on(addr.clone()).map_err(|e| {
            let err = format!("Error listening on {addr:?}: {e:?}");
            error!("{err}");
            err
        }).unwrap();
        // println!("Listening on {addr:?}");
    }

    println!("Peer Id: {:?}", swarm.local_peer_id());

    println!("Multiaddrs: {:?}", swarm.external_addresses().collect::<Vec<_>>());

    println!("Network Info: {:?}", swarm.network_info());

    loop {
        match swarm.next().await.expect("Infinite Stream.") {
            SwarmEvent::Behaviour(event) => {
                info!("Behaviour Event: {event:?}");
                match event {
                   NodeBehaviourEvent::Relay(relay_event) => {
                       info!("Relay event: {relay_event:?}");
                       match relay_event {
                           Event::ReservationReqAccepted { src_peer_id, renewed, .. } => {
                               info!("Relay reservation accepted from {src_peer_id}, renewed: {renewed}");
                               metrics.connection_established(src_peer_id);
                           }
                           Event::CircuitReqAccepted { src_peer_id, dst_peer_id, .. } => {
                               info!("Circuit established: {src_peer_id} <-> {dst_peer_id}");
                               metrics.message_relayed(&src_peer_id);
                           }
                           _ => {}
                       }
                   }
                   NodeBehaviourEvent::Identify(id_event) => {

                    if let identify::Event::Received { info: identify::Info { listen_addrs: _, observed_addr, .. }, .. } = id_event {
                        info!("Received identify message from Observed Address: {observed_addr:?}");
                        swarm.add_external_address(observed_addr.clone());

                        // for listen_addr in listen_addrs {
                        //     swarm.add_external_address(listen_addr.clone());
                        // }
                    }

                   }
                   NodeBehaviourEvent::Ping(_event) => {

                   }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let addr = address.with_p2p(swarm.local_peer_id().clone()).unwrap();
                println!("Listening on {addr:?}");
                metrics.add_relay_address(addr.to_string());
            }

            SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {
                info!("Incoming connection: {connection_id} -> {local_addr} -> {send_back_addr}");
            }

            SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => {
                info!("Connection established: {peer_id} -> {connection_id} -> {endpoint:?} -> {num_established} -> {concurrent_dial_errors:?} -> {established_in:?}");
                metrics.connection_established(peer_id);
            }

            SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {
                info!("Connection closed: {peer_id} -> {connection_id} -> {endpoint:?} -> {num_established} -> {cause:?}");
                if num_established == 0 {
                    metrics.connection_closed(&peer_id);
                }
            }

            _ => {}
        }
    }
}


