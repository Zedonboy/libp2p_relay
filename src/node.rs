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

use libp2p::{noise, relay::{self, Behaviour, Config}, tcp, yamux, Swarm, SwarmBuilder, Transport};

pub async fn create_swarm() -> Result<Swarm<Behaviour>, Box<dyn std::error::Error>> {
    let swarm = SwarmBuilder::with_new_identity()
    .with_tokio()
    .with_tcp(
        tcp::Config::default(),
        (libp2p::tls::Config::new, noise::Config::new),
        yamux::Config::default,
    )?
    .with_dns()?
    .with_websocket(
        (libp2p::tls::Config::new, libp2p::noise::Config::new),
        libp2p::yamux::Config::default,
    ).await?
    .with_behaviour(|key| relay::Behaviour::new(key.public().to_peer_id(), Config::default()))?
    .build();

    Ok(swarm)

}