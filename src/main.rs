use std::{env, net::{Ipv4Addr, Ipv6Addr}};

use libp2p::{futures::StreamExt, identify, multiaddr::Protocol, relay::Event, swarm::SwarmEvent, Multiaddr, Swarm};
use tracing::{error, info};

use crate::node::NodeBehaviourEvent;
mod node;

fn get_listen_addrs() -> Vec<String> {

    vec![
        format!("/ip4/0.0.0.0/tcp/3000/ws"),
        format!("/ip6/::/tcp/3000/ws"),
        format!("/ip4/0.0.0.0/tcp/3000"),
        format!("/ip6/::/tcp/3000"),
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

    println!("Network Info: {:?}", swarm.network_info());

    loop {
        match swarm.next().await.expect("Infinite Stream.") {
            SwarmEvent::Behaviour(event) => {
                info!("Behaviour Event: {event:?}");
                match event {
                   NodeBehaviourEvent::Relay(event) => {

                   }
                   NodeBehaviourEvent::Identify(id_event) => {

                    if let identify::Event::Received { info: identify::Info { listen_addrs, observed_addr, .. }, .. } = id_event {
                        info!("Received identify message from Observed Address: {observed_addr:?}");
                        swarm.add_external_address(observed_addr.clone());

                        // for listen_addr in listen_addrs {
                        //     swarm.add_external_address(listen_addr.clone());
                        // }
                    }

                   }
                   NodeBehaviourEvent::Ping(event) => {

                   }
                    _ => {
                        info!("Event: {event:?}");
                    }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let addr = address.with_p2p(swarm.local_peer_id().clone()).unwrap();
                println!("Listening on {addr:?}");
            }

            SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {
                info!("Incoming connection: {connection_id} -> {local_addr} -> {send_back_addr}");
            }

            SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => {
                info!("Connection established: {peer_id} -> {connection_id} -> {endpoint:?} -> {num_established} -> {concurrent_dial_errors:?} -> {established_in:?}");
            }

            _ => {}
        }
    }
}


