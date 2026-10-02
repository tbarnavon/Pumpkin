//! Login queries that plugins register with `context.register-login-query`.
//!
//! Like Fabric's `ServerLoginNetworking`: sent to every client before login success, answers
//! reported with `PlayerLoginQueryResponseEvent`.

use std::sync::Mutex;

/// Message ids for plugin queries start here, clear of the proxy forwarding request.
pub const FIRST_MESSAGE_ID: i32 = 0x5000;

static QUERIES: Mutex<Vec<(String, Vec<u8>)>> = Mutex::new(Vec::new());

/// Registers or replaces the query sent on `channel`.
pub fn register(channel: String, payload: Vec<u8>) {
    let mut queries = QUERIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(entry) = queries.iter_mut().find(|(c, _)| *c == channel) {
        entry.1 = payload;
    } else {
        queries.push((channel, payload));
    }
}

static CONFIGURATION: Mutex<Vec<(String, Vec<u8>)>> = Mutex::new(Vec::new());

/// Registers or replaces the payload sent on `channel` in the configuration phase.
pub fn register_configuration(channel: String, payload: Vec<u8>) {
    let mut payloads = CONFIGURATION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(entry) = payloads.iter_mut().find(|(c, _)| *c == channel) {
        entry.1 = payload;
    } else {
        payloads.push((channel, payload));
    }
}

/// Every configuration-phase payload, in registration order.
pub fn configuration() -> Vec<(String, Vec<u8>)> {
    CONFIGURATION
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Every registered query, in registration order.
pub fn all() -> Vec<(String, Vec<u8>)> {
    QUERIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}
