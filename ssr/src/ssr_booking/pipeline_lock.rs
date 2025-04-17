use dashmap::DashMap;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct PipelineLockManager {
    // Use DashMap for concurrent access without needing explicit locking
    active_pipelines: Arc<DashMap<String, bool>>,
}

impl PipelineLockManager {
    pub fn new() -> Self {
        Self {
            active_pipelines: Arc::new(DashMap::new()),
        }
    }

    // Create a unique key from payment_id and order_id
    fn create_lock_key(payment_id: Option<&str>, order_id: &str) -> String {
        format!("{}:{}", payment_id.unwrap_or("none"), order_id)
    }

    // Try to acquire a lock for a pipeline
    pub fn try_acquire_lock(&self, payment_id: Option<&str>, order_id: &str) -> bool {
        let key = Self::create_lock_key(payment_id, order_id);
        debug!("Attempting to acquire lock for key: {}", key);
        // Only insert if key doesn't exist
        let acquired = self.active_pipelines.insert(key.clone(), true).is_none();
        if acquired {
            debug!("Lock acquired for key: {}", key);
        } else {
            debug!("Lock already exists for key: {}", key);
        }
        acquired
    }

    // Release the lock when pipeline completes
    pub fn release_lock(&self, payment_id: Option<&str>, order_id: &str) {
        let key = Self::create_lock_key(payment_id, order_id);
        self.active_pipelines.remove(&key);
        debug!("Lock released for key: {}", key);
    }

    // Check if a lock exists for a given key
    pub fn has_lock(&self, payment_id: Option<&str>, order_id: &str) -> bool {
        let key = Self::create_lock_key(payment_id, order_id);
        self.active_pipelines.contains_key(&key)
    }

    // Get a list of all currently locked keys
    pub fn list_active_locks(&self) -> Vec<String> {
        self.active_pipelines
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

impl Default for PipelineLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod pipeline_lock_tests {
    use super::*;
    use std::sync::Arc;
    use tokio;

    #[test]
    fn test_lock_acquisition_and_release() {
        let manager = PipelineLockManager::new();

        // First acquisition should succeed
        assert!(manager.try_acquire_lock(Some("pay_123"), "order_456"));

        // Second acquisition should fail
        assert!(!manager.try_acquire_lock(Some("pay_123"), "order_456"));

        // Release the lock
        manager.release_lock(Some("pay_123"), "order_456");

        // Should be able to acquire again
        assert!(manager.try_acquire_lock(Some("pay_123"), "order_456"));
    }

    #[test]
    fn test_different_keys_can_be_acquired() {
        let manager = PipelineLockManager::new();

        // Acquire first lock
        assert!(manager.try_acquire_lock(Some("pay_123"), "order_456"));

        // Different payment_id, same order_id
        assert!(manager.try_acquire_lock(Some("pay_789"), "order_456"));

        // Same payment_id, different order_id
        assert!(manager.try_acquire_lock(Some("pay_123"), "order_789"));

        // Both different
        assert!(manager.try_acquire_lock(Some("pay_999"), "order_999"));
    }

    #[test]
    fn test_none_payment_id_lock() {
        let manager = PipelineLockManager::new();

        // Acquire lock with None payment_id
        assert!(manager.try_acquire_lock(None, "order_456"));

        // Try to acquire same lock
        assert!(!manager.try_acquire_lock(None, "order_456"));

        // Release and try again
        manager.release_lock(None, "order_456");
        assert!(manager.try_acquire_lock(None, "order_456"));
    }

    #[tokio::test]
    async fn test_concurrent_lock_operations() {
        let manager = Arc::new(PipelineLockManager::new());
        let manager_clone = manager.clone();

        // Spawn a task that acquires and holds a lock
        let handle = tokio::spawn(async move {
            assert!(manager_clone.try_acquire_lock(Some("pay_123"), "order_456"));
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            manager_clone.release_lock(Some("pay_123"), "order_456");
        });

        // Try to acquire the same lock while it's held
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(!manager.try_acquire_lock(Some("pay_123"), "order_456"));

        // Wait for the first task to complete
        handle.await.unwrap();

        // Should be able to acquire the lock now
        assert!(manager.try_acquire_lock(Some("pay_123"), "order_456"));
    }

    #[test]
    fn test_has_lock() {
        let manager = PipelineLockManager::new();

        // Initially no lock
        assert!(!manager.has_lock(Some("pay_123"), "order_456"));

        // Acquire lock
        manager.try_acquire_lock(Some("pay_123"), "order_456");
        assert!(manager.has_lock(Some("pay_123"), "order_456"));

        // Release lock
        manager.release_lock(Some("pay_123"), "order_456");
        assert!(!manager.has_lock(Some("pay_123"), "order_456"));
    }

    #[test]
    fn test_list_active_locks() {
        let manager = PipelineLockManager::new();

        // Initially empty
        assert!(manager.list_active_locks().is_empty());

        // Add some locks
        manager.try_acquire_lock(Some("pay_123"), "order_456");
        manager.try_acquire_lock(Some("pay_789"), "order_789");

        let active_locks = manager.list_active_locks();
        assert_eq!(active_locks.len(), 2);
        assert!(active_locks.contains(&PipelineLockManager::create_lock_key(
            Some("pay_123"),
            "order_456"
        )));
        assert!(active_locks.contains(&PipelineLockManager::create_lock_key(
            Some("pay_789"),
            "order_789"
        )));

        // Release one lock
        manager.release_lock(Some("pay_123"), "order_456");
        let active_locks = manager.list_active_locks();
        assert_eq!(active_locks.len(), 1);
        assert!(active_locks.contains(&PipelineLockManager::create_lock_key(
            Some("pay_789"),
            "order_789"
        )));
    }
}
