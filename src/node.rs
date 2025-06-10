// Copyright 2025 Declan Nnadozie
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     https://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use libp2p::{identify, noise, ping::{self, Config}, relay::{self, Behaviour}, swarm::NetworkBehaviour, tcp, yamux, Swarm, SwarmBuilder, Transport};

pub async fn create_swarm() -> Result<Swarm<NodeBehaviour>, Box<dyn std::error::Error>> {
    let swarm = SwarmBuilder::with_new_identity()
    .with_tokio()
    .with_websocket(
        (libp2p::tls::Config::new, libp2p::noise::Config::new),
        libp2p::yamux::Config::default,
    ).await?
    .with_behaviour(|key| NodeBehaviour {
        relay: relay::Behaviour::new(key.public().to_peer_id(), libp2p::relay::Config::default()),
        ping: ping::Behaviour::new(Config::default()),
        identify: identify::Behaviour::new(libp2p::identify::Config::new(
            "/fusion/1.0.0".to_string(), key.public())),
    })?
    .build();

    Ok(swarm)

}

#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    pub relay: relay::Behaviour,
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
}