use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct OtpEntry {
    pub otp: String,
    pub created_at: Instant,
    pub expires_in: Duration,
}

impl OtpEntry {
    pub fn new(otp: String) -> Self {
        Self {
            otp,
            created_at: Instant::now(),
            expires_in: Duration::from_secs(10 * 60), // 10 minutes
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.expires_in
    }
}

#[derive(Debug, Clone)]
pub struct OtpStorage {
    storage: Arc<Mutex<HashMap<String, OtpEntry>>>,
}

impl Default for OtpStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl OtpStorage {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store OTP with 10-minute expiration
    pub fn store_otp(&self, booking_id: &str, otp: String) -> Result<(), String> {
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Failed to acquire lock for OTP storage: {}", e))?;

        let entry = OtpEntry::new(otp.clone());
        storage.insert(booking_id.to_string(), entry);

        info!(
            "OTP stored for booking_id: {} (expires in 10 minutes)",
            booking_id
        );
        debug!("OTP value stored: {}", otp);

        Ok(())
    }

    /// Verify OTP and remove if valid (one-time use)
    pub fn verify_otp(&self, booking_id: &str, provided_otp: &str) -> Result<bool, String> {
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Failed to acquire lock for OTP verification: {}", e))?;

        // Clean up expired entries during verification (lazy cleanup)
        self.cleanup_expired_entries(&mut storage);

        match storage.get(booking_id) {
            Some(entry) => {
                if entry.is_expired() {
                    storage.remove(booking_id);
                    warn!("OTP expired for booking_id: {}", booking_id);
                    return Ok(false);
                }

                let is_valid = entry.otp == provided_otp;

                if is_valid {
                    // Remove OTP after successful verification (one-time use)
                    storage.remove(booking_id);
                    info!(
                        "OTP verified successfully and removed for booking_id: {}",
                        booking_id
                    );
                } else {
                    warn!("Invalid OTP provided for booking_id: {}", booking_id);
                    debug!("Expected: {}, Provided: {}", entry.otp, provided_otp);
                }

                Ok(is_valid)
            }
            None => {
                warn!("No OTP found for booking_id: {}", booking_id);
                Ok(false)
            }
        }
    }

    /// Generate a random 6-digit numeric OTP
    pub fn generate_6_digit_otp() -> String {
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(100000..=999999))
    }

    /// Clean up expired entries (called during verification)
    fn cleanup_expired_entries(&self, storage: &mut HashMap<String, OtpEntry>) {
        let initial_count = storage.len();
        storage.retain(|booking_id, entry| {
            if entry.is_expired() {
                debug!("Removing expired OTP for booking_id: {}", booking_id);
                false
            } else {
                true
            }
        });

        let final_count = storage.len();
        if initial_count > final_count {
            info!(
                "Cleaned up {} expired OTP entries",
                initial_count - final_count
            );
        }
    }

    /// Get current storage size (for monitoring/debugging)
    pub fn storage_size(&self) -> Result<usize, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Failed to acquire lock for storage size check: {}", e))?;
        Ok(storage.len())
    }

    /// Clear all OTPs (for testing purposes)
    #[cfg(test)]
    pub fn clear_all(&self) -> Result<(), String> {
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Failed to acquire lock for clearing storage: {}", e))?;
        storage.clear();
        Ok(())
    }
}

// Global OTP storage instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_OTP_STORAGE: OtpStorage = OtpStorage::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_generate_6_digit_otp() {
        let otp = OtpStorage::generate_6_digit_otp();
        assert_eq!(otp.len(), 6);
        assert!(otp.chars().all(|c| c.is_ascii_digit()));

        // Test that it generates different OTPs
        let otp2 = OtpStorage::generate_6_digit_otp();
        // This could theoretically fail 1 in 900,000 times, but very unlikely
        assert_ne!(otp, otp2);
    }

    #[test]
    fn test_store_and_verify_otp() {
        let storage = OtpStorage::new();
        let booking_id = "test_booking_123";
        let otp = "123456";

        // Store OTP
        assert!(storage.store_otp(booking_id, otp.to_string()).is_ok());

        // Verify correct OTP
        assert_eq!(storage.verify_otp(booking_id, otp).unwrap(), true);

        // OTP should be removed after successful verification
        assert_eq!(storage.verify_otp(booking_id, otp).unwrap(), false);
    }

    #[test]
    fn test_verify_wrong_otp() {
        let storage = OtpStorage::new();
        let booking_id = "test_booking_456";
        let correct_otp = "123456";
        let wrong_otp = "654321";

        // Store OTP
        assert!(storage
            .store_otp(booking_id, correct_otp.to_string())
            .is_ok());

        // Verify wrong OTP
        assert_eq!(storage.verify_otp(booking_id, wrong_otp).unwrap(), false);

        // Correct OTP should still be available
        assert_eq!(storage.verify_otp(booking_id, correct_otp).unwrap(), true);
    }

    #[test]
    fn test_verify_nonexistent_booking() {
        let storage = OtpStorage::new();
        let result = storage.verify_otp("nonexistent_booking", "123456");
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_otp_expiration() {
        let storage = OtpStorage::new();
        let booking_id = "test_booking_expire";

        // Create an entry that's already expired
        let mut storage_map = storage.storage.lock().unwrap();
        let expired_entry = OtpEntry {
            otp: "123456".to_string(),
            created_at: Instant::now() - StdDuration::from_secs(11 * 60), // 11 minutes ago
            expires_in: Duration::from_secs(10 * 60),                     // 10 minute expiry
        };
        storage_map.insert(booking_id.to_string(), expired_entry);
        drop(storage_map);

        // Verification should fail for expired OTP
        assert_eq!(storage.verify_otp(booking_id, "123456").unwrap(), false);
    }

    #[test]
    fn test_storage_size() {
        let storage = OtpStorage::new();

        assert_eq!(storage.storage_size().unwrap(), 0);

        storage.store_otp("booking1", "123456".to_string()).unwrap();
        storage.store_otp("booking2", "654321".to_string()).unwrap();

        assert_eq!(storage.storage_size().unwrap(), 2);

        // Verify one OTP (removes it)
        storage.verify_otp("booking1", "123456").unwrap();
        assert_eq!(storage.storage_size().unwrap(), 1);
    }

    #[test]
    fn test_clear_all() {
        let storage = OtpStorage::new();

        storage.store_otp("booking1", "123456".to_string()).unwrap();
        storage.store_otp("booking2", "654321".to_string()).unwrap();
        assert_eq!(storage.storage_size().unwrap(), 2);

        storage.clear_all().unwrap();
        assert_eq!(storage.storage_size().unwrap(), 0);
    }
}
