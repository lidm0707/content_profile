use crate::models::{Content, ContentRequest};
use crate::services::{LocalStorageService, SupabaseService};
use std::time::Duration;
use tracing::{info, trace, warn};

/// Sync direction enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Pull,
    Push,
    Bidirectional,
}

/// Sync service for syncing data between local and remote storage
#[derive(Clone)]
pub struct SyncService {
    local_service: LocalStorageService,
    remote_service: SupabaseService,
}

impl SyncService {
    /// Creates a new sync service instance
    pub fn new() -> Self {
        SyncService {
            local_service: LocalStorageService::new(),
            remote_service: SupabaseService::new(),
        }
    }

    /// Syncs content from remote to local (pull)
    pub async fn sync_pull(&self) -> Result<(), String> {
        match self.remote_service.get_all_content().await {
            Ok(remote_contents) => {
                self.local_service
                    .sync_content(remote_contents)
                    .map_err(|e| format!("Failed to sync to local storage: {}", e))?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Syncs content from local to remote (push)
    pub async fn sync_push(&self) -> Result<(), String> {
        match self.remote_service.get_all_content().await {
            Ok(remote_contents) => {
                let remote_ids: Vec<i32> = remote_contents.iter().filter_map(|c| c.id).collect();

                let unsynced = self
                    .local_service
                    .get_unsynced_content(&remote_ids)
                    .map_err(|e| format!("Failed to get unsynced content: {}", e))?;

                for content in unsynced {
                    let content_request: ContentRequest = content.clone().into();
                    if let Some(local_id) = content.id {
                        trace!(
                            "Pushing local content '{}' to remote (local_id: {})",
                            content.title, local_id
                        );
                        match self.remote_service.create_content(content_request).await {
                            Ok(remote_content) => {
                                if let Some(remote_id) = remote_content.id {
                                    info!(
                                        "Successfully synced local content '{}' to remote_id: {}",
                                        content.title, remote_id
                                    );
                                    let _ = self.local_service.mark_as_synced(local_id, remote_id);
                                } else {
                                    warn!(
                                        "Remote content created but no ID returned for '{}'",
                                        content.title
                                    );
                                }
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to sync local content '{}' to remote: {}",
                                    content.title, e
                                );
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Syncs content bidirectionally
    pub async fn sync_bidirectional(&self) -> Result<(), String> {
        match self.remote_service.get_all_content().await {
            Ok(remote_contents) => {
                let remote_ids: Vec<i32> = remote_contents.iter().filter_map(|c| c.id).collect();

                let synced_local = self
                    .local_service
                    .sync_content(remote_contents.clone())
                    .map_err(|e| format!("Failed to sync to local storage: {}", e))?;

                let unsynced: Vec<Content> = synced_local
                    .iter()
                    .filter(|c| {
                        if let Some(id) = c.id {
                            !remote_ids.contains(&id)
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect();

                for content in unsynced {
                    let content_request: ContentRequest = content.clone().into();
                    if let Some(local_id) = content.id {
                        trace!(
                            "Pushing local content '{}' to remote (local_id: {})",
                            content.title, local_id
                        );
                        match self.remote_service.create_content(content_request).await {
                            Ok(remote_content) => {
                                if let Some(remote_id) = remote_content.id {
                                    info!(
                                        "Successfully synced local content '{}' to remote_id: {}",
                                        content.title, remote_id
                                    );
                                    let _ = self.local_service.mark_as_synced(local_id, remote_id);
                                } else {
                                    warn!(
                                        "Remote content created but no ID returned for '{}'",
                                        content.title
                                    );
                                }
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to sync local content '{}' to remote: {}",
                                    content.title, e
                                );
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn auto_sync(&self, _duration: Duration, _direction: SyncDirection) {
        unimplemented!("auto_sync not implemented")
    }
}

impl Default for SyncService {
    fn default() -> Self {
        Self::new()
    }
}
