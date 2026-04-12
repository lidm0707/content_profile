use dioxus::signals::{ReadableExt, Signal, WritableExt};

use crate::services::{SyncDirection, SyncService};
use std::time::Duration;

/// Sync context for managing data synchronization across the app
#[derive(Clone)]
pub struct SyncContext {
    sync_service: Signal<SyncService>,
}

impl SyncContext {
    /// Creates a new SyncContext
    pub fn new() -> Self {
        SyncContext {
            sync_service: Signal::new(SyncService::new()),
        }
    }

    /// Gets the sync service
    pub fn sync_service(&self) -> SyncService {
        self.sync_service.cloned()
    }

    /// Syncs content from remote to local (pull)
    pub async fn sync_pull(&mut self) -> Result<(), String> {
        self.sync_service.write().sync_pull().await
    }

    /// Syncs content from local to remote (push)
    pub async fn sync_push(&mut self) -> Result<(), String> {
        self.sync_service.write().sync_push().await
    }

    /// Syncs content bidirectionally
    pub async fn sync_bidirectional(&mut self) -> Result<(), String> {
        self.sync_service.write().sync_bidirectional().await
    }

    /// Auto-syncs content at specified interval
    pub async fn auto_sync(&self, duration: Duration, direction: SyncDirection) {
        self.sync_service
            .read()
            .auto_sync(duration, direction)
            .await;
    }
}

impl Default for SyncContext {
    fn default() -> Self {
        Self::new()
    }
}
