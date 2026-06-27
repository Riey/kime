//! `org.freedesktop.IBus.Factory` implementation.
//!
//! The factory is served at `/org/freedesktop/IBus/Factory`. When the daemon
//! activates the engine it calls `CreateEngine`, for which we spin up a new
//! [`IBusEngine`] object served at a unique path and return that path.

use std::sync::Arc;

use kime_engine_core::Config;
use zbus::{interface, zvariant::OwnedObjectPath, Connection};

use crate::engine::IBusEngine;

pub struct IBusFactory {
    conn: Connection,
    config: Arc<Config>,
    next_id: u64,
}

impl IBusFactory {
    pub fn new(conn: Connection, config: Arc<Config>) -> Self {
        Self {
            conn,
            config,
            next_id: 0,
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Factory")]
impl IBusFactory {
    async fn create_engine(&mut self, engine_name: String) -> zbus::fdo::Result<OwnedObjectPath> {
        let id = self.next_id;
        self.next_id += 1;
        let path = format!("/org/freedesktop/IBus/Engine/Kime/{}", id);
        log::info!("CreateEngine({}) -> {}", engine_name, path);

        let engine = IBusEngine::new(self.conn.clone(), path.clone(), self.config.clone());

        if let Err(e) = self.conn.object_server().at(path.as_str(), engine).await {
            return Err(zbus::fdo::Error::Failed(format!(
                "Failed to register engine object: {e}"
            )));
        }

        OwnedObjectPath::try_from(path)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid object path: {e}")))
    }
}
