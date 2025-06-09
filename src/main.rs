use libp2p::{futures::StreamExt, relay::Event, swarm::SwarmEvent};
use tracing::info;
mod node;

const listen_addr_list : &[&str] = &[
    "/ip4/0.0.0.0/tcp/3000/ws",      // WebSocket HTTP
    "/ip6/::/tcp/3000/ws",           // WebSocket IPv6 HTTP
    "/ip4/0.0.0.0/tcp/3000",
    "/ip6/::/tcp/3000",
];

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


