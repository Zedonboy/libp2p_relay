use std::env;

use libp2p::{futures::StreamExt, relay::Event, swarm::SwarmEvent, Multiaddr};
use tracing::{error, info};
mod node;

fn get_listen_addrs() -> Vec<String> {
    // Read port from env or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    vec![
        format!("/ip4/0.0.0.0/tcp/{}/ws", port),
        format!("/ip6/::/tcp/{}/ws", port),
        format!("/ip4/0.0.0.0/tcp/{}", port),
        format!("/ip6/::/tcp/{}", port),
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

    loop {
        match swarm.next().await.expect("Infinite Stream.") {
            SwarmEvent::Behaviour(event) => {
                match event {
                    Event::CircuitClosed { src_peer_id, dst_peer_id, error } => {
                        info!("Circuit closed: {src_peer_id} -> {dst_peer_id} with error: {error:?}");
                    }

                    Event::CircuitReqAccepted { src_peer_id, dst_peer_id } => {
                        info!("Circuit request accepted: {src_peer_id} -> {dst_peer_id}");
                    }

                    Event::ReservationReqAccepted { src_peer_id, renewed } => {
                        info!("Reservation request accepted: {src_peer_id} -> {renewed}");
                    }

                    Event::ReservationReqDenied { src_peer_id } => {
                        info!("Reservation request denied: {src_peer_id}");
                    }
                    _ => {
                        info!("Event: {event:?}");
                    }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}");
            }
            _ => {}
        }
    }
}


