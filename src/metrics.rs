use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use libp2p::PeerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    pub peer_id: String,
    pub connected_at: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_relayed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_bytes_transferred_24h: u64,
    pub total_messages_relayed_24h: u64,
    pub relay_addresses: Vec<String>,
    pub uptime: Duration,
    pub connections: Vec<ConnectionMetrics>,
}

#[derive(Debug, Clone)]
pub struct MetricsCollector {
    connections: Arc<Mutex<HashMap<PeerId, ConnectionMetrics>>>,
    connection_history: Arc<Mutex<Vec<DateTime<Utc>>>>,
    start_time: DateTime<Utc>,
    relay_addresses: Arc<Mutex<Vec<String>>>,
    peer_id: String,
}

impl MetricsCollector {
    pub fn new(peer_id: String) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            connection_history: Arc::new(Mutex::new(Vec::new())),
            start_time: Utc::now(),
            relay_addresses: Arc::new(Mutex::new(Vec::new())),
            peer_id,
        }
    }

    pub fn add_relay_address(&self, address: String) {
        if let Ok(mut addrs) = self.relay_addresses.lock() {
            if !addrs.contains(&address) {
                addrs.push(address);
            }
        }
    }

    pub fn get_peer_id(&self) -> String {
        self.peer_id.clone()
    }

    pub fn connection_established(&self, peer_id: PeerId) {
        let now = Utc::now();
        if let Ok(mut connections) = self.connections.lock() {
            connections.insert(peer_id, ConnectionMetrics {
                peer_id: peer_id.to_string(),
                connected_at: now,
                bytes_sent: 0,
                bytes_received: 0,
                messages_relayed: 0,
            });
        }
        if let Ok(mut history) = self.connection_history.lock() {
            history.push(now);
        }
    }

    pub fn connection_closed(&self, peer_id: &PeerId) {
        if let Ok(mut connections) = self.connections.lock() {
            connections.remove(peer_id);
        }
    }

    pub fn bytes_transferred(&self, peer_id: &PeerId, sent: u64, received: u64) {
        if let Ok(mut connections) = self.connections.lock() {
            if let Some(metrics) = connections.get_mut(peer_id) {
                metrics.bytes_sent += sent;
                metrics.bytes_received += received;
            }
        }
    }

    pub fn message_relayed(&self, peer_id: &PeerId) {
        if let Ok(mut connections) = self.connections.lock() {
            if let Some(metrics) = connections.get_mut(peer_id) {
                metrics.messages_relayed += 1;
            }
        }
    }

    pub fn get_metrics(&self) -> RelayMetrics {
        let now = Utc::now();
        let twenty_four_hours_ago = now - Duration::hours(24);
        
        let connections = self.connections.lock().unwrap();
        let connection_history = self.connection_history.lock().unwrap();
        let relay_addresses = self.relay_addresses.lock().unwrap().clone();
        
        // Filter connection history to only include connections from last 24 hours
        let total_connections = connection_history.iter()
            .filter(|&timestamp| *timestamp >= twenty_four_hours_ago)
            .count() as u64;
        
        let active_connections = connections.len() as u64;
        
        let mut total_bytes_24h = 0;
        let mut total_messages_24h = 0;
        let mut active_conns = Vec::new();
        
        for conn in connections.values() {
            if conn.connected_at >= twenty_four_hours_ago {
                total_bytes_24h += conn.bytes_sent + conn.bytes_received;
                total_messages_24h += conn.messages_relayed;
            }
            active_conns.push(conn.clone());
        }
        
        RelayMetrics {
            total_connections,
            active_connections,
            total_bytes_transferred_24h: total_bytes_24h,
            total_messages_relayed_24h: total_messages_24h,
            relay_addresses,
            uptime: now - self.start_time,
            connections: active_conns,
        }
    }
}