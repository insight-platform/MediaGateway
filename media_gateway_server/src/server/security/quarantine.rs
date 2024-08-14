use std::time::Duration;

use anyhow::{anyhow, Result};
use log::warn;
use mockall::automock;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

use crate::server::configuration::BasicAuthConfiguration;
use crate::server::service::cache::{Cache, CacheUsageFactory, LruTtlSet};

#[automock]
pub trait AuthQuarantine {
    fn register_failure(&self, username: &str);

    fn register_success(&self, username: &str);

    fn in_quarantine(&self, username: &str) -> bool;
}

pub struct AuthQuarantineImpl {
    failed_attempt_limit: u32,
    failed_attempt_cache: Cache<String, u32>,
    quarantine: LruTtlSet<String>,
    mutex: Mutex<()>,
}

impl AuthQuarantineImpl {
    pub fn new(
        failed_attempt_limit: u32,
        failed_attempt_cache: Cache<String, u32>,
        quarantine: LruTtlSet<String>,
    ) -> Self {
        Self {
            failed_attempt_limit,
            failed_attempt_cache,
            quarantine,
            mutex: Mutex::new(()),
        }
    }
}

impl AuthQuarantine for AuthQuarantineImpl {
    fn register_failure(&self, username: &str) {
        let _unused = self.mutex.lock();
        let username = username.to_string();
        if self.quarantine.contains(&username) {
            // by usage logic this case should never happen
            warn!("Failure is registered while being in quarantine");
            return;
        }
        let failed_attempts = self
            .failed_attempt_cache
            .get(&username)
            .map_or(1, |e| e + 1);
        if failed_attempts == self.failed_attempt_limit {
            self.failed_attempt_cache.pop(&username);
            self.quarantine.add(username);
        } else {
            self.failed_attempt_cache.push(username, failed_attempts);
        }
    }

    fn register_success(&self, username: &str) {
        let _unused = self.mutex.lock();
        let username = username.to_string();
        if self.quarantine.contains(&username) {
            // by usage logic this case should never happen
            warn!("Success is registered while being in quarantine");
            return;
        }
        self.failed_attempt_cache.pop(&username);
    }

    fn in_quarantine(&self, username: &str) -> bool {
        let _unused = self.mutex.lock();
        self.quarantine.contains(username)
    }
}

pub struct NoOpAuthQuarantine {}

impl AuthQuarantine for NoOpAuthQuarantine {
    fn register_failure(&self, _username: &str) {}

    fn register_success(&self, _username: &str) {}

    fn in_quarantine(&self, _username: &str) -> bool {
        false
    }
}

pub struct AuthQuarantineFactory {}

impl AuthQuarantineFactory {
    pub fn from(
        configuration: &BasicAuthConfiguration,
        runtime: &Runtime,
    ) -> Result<Box<dyn AuthQuarantine + Send + Sync>> {
        if let Some(quarantine_config) = &configuration.quarantine {
            if quarantine_config.period == Duration::ZERO {
                return Err(anyhow!("Invalid quarantine period: zero"));
            }
            if quarantine_config.failed_attempt_limit == 0 {
                return Err(anyhow!("Invalid quarantine failed_attempt_limit: zero"));
            }
            let failed_attempt_usage_tracker = CacheUsageFactory::from(
                configuration.cache.usage.as_ref(),
                "auth failed attempt".to_string(),
                runtime,
            );
            let quarantine_usage_tracker = CacheUsageFactory::from(
                configuration.cache.usage.as_ref(),
                "auth quarantine".to_string(),
                runtime,
            );
            Ok(Box::new(AuthQuarantineImpl::new(
                quarantine_config.failed_attempt_limit,
                Cache::new(configuration.cache.size, failed_attempt_usage_tracker),
                LruTtlSet::new(
                    configuration.cache.size,
                    quarantine_config.period,
                    quarantine_usage_tracker,
                ),
            )))
        } else {
            Ok(Box::new(NoOpAuthQuarantine {}))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    use crate::server::security::quarantine::{AuthQuarantine, AuthQuarantineImpl};
    use crate::server::service::cache::{Cache, LruTtlSet, NoOpCacheUsageTracker};

    const USERNAME: &str = "user";

    #[test]
    pub fn in_quarantine_empty() {
        let auth_quarantine = new_auth_quarantine(10, Duration::from_secs(1));

        let result = auth_quarantine.in_quarantine(USERNAME);

        assert_eq!(result, false);
    }

    #[test]
    pub fn in_quarantine_existing_value() {
        let cache = new_cache();
        let lru_ttl_set = new_lru_ttl_set(Duration::from_secs(1));
        lru_ttl_set.add(USERNAME.to_string());
        let auth_quarantine = AuthQuarantineImpl::new(10, cache, lru_ttl_set);

        let result = auth_quarantine.in_quarantine(USERNAME);

        assert_eq!(result, true);
    }

    #[test]
    pub fn in_quarantine_another_value() {
        let cache = new_cache();
        let lru_ttl_set = new_lru_ttl_set(Duration::from_secs(1));
        lru_ttl_set.add("abc".to_string());
        let auth_quarantine = AuthQuarantineImpl::new(10, cache, lru_ttl_set);

        let result = auth_quarantine.in_quarantine(USERNAME);

        assert_eq!(result, false);
    }

    #[test]
    pub fn in_quarantine_expired_value() {
        let duration = Duration::from_millis(10);
        let cache = new_cache();
        let lru_ttl_set = new_lru_ttl_set(duration);
        lru_ttl_set.add(USERNAME.to_string());
        let auth_quarantine = AuthQuarantineImpl::new(10, cache, lru_ttl_set);

        let result = auth_quarantine.in_quarantine(USERNAME);

        assert_eq!(result, true);

        sleep(duration);

        let result = auth_quarantine.in_quarantine(USERNAME);

        assert_eq!(result, false);
    }

    #[test]
    pub fn register_failure_in_quarantine() {
        let cache = new_cache();
        let lru_ttl_set = new_lru_ttl_set(Duration::from_secs(1));
        lru_ttl_set.add(USERNAME.to_string());
        let auth_quarantine = AuthQuarantineImpl::new(10, cache, lru_ttl_set);

        auth_quarantine.register_failure(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), true);
    }

    #[test]
    pub fn register_failure_first_attempt() {
        let auth_quarantine = new_auth_quarantine(10, Duration::from_secs(1));

        auth_quarantine.register_failure(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);
    }

    #[test]
    pub fn register_failure_last_attempt() {
        let auth_quarantine = new_auth_quarantine(2, Duration::from_secs(1));

        auth_quarantine.register_failure(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);

        auth_quarantine.register_failure(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), true);
    }

    #[test]
    pub fn register_success_in_quarantine() {
        let cache = new_cache();
        let lru_ttl_set = new_lru_ttl_set(Duration::from_secs(1));
        lru_ttl_set.add(USERNAME.to_string());
        let auth_quarantine = AuthQuarantineImpl::new(10, cache, lru_ttl_set);

        auth_quarantine.register_success(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), true);
    }

    #[test]
    pub fn register_success_first_attempt() {
        let auth_quarantine = new_auth_quarantine(2, Duration::from_secs(1));

        auth_quarantine.register_failure(USERNAME);
        auth_quarantine.register_success(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);

        auth_quarantine.register_failure(USERNAME);
        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);

        auth_quarantine.register_failure(USERNAME);
        assert_eq!(auth_quarantine.in_quarantine(USERNAME), true);
    }

    #[test]
    pub fn register_success_penultimate_attempt() {
        let auth_quarantine = new_auth_quarantine(3, Duration::from_secs(1));

        auth_quarantine.register_failure(USERNAME);
        auth_quarantine.register_failure(USERNAME);
        auth_quarantine.register_success(USERNAME);

        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);

        auth_quarantine.register_failure(USERNAME);
        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);

        auth_quarantine.register_failure(USERNAME);
        assert_eq!(auth_quarantine.in_quarantine(USERNAME), false);

        auth_quarantine.register_failure(USERNAME);
        assert_eq!(auth_quarantine.in_quarantine(USERNAME), true);
    }

    fn new_auth_quarantine(failed_attempt_limit: u32, ttl: Duration) -> AuthQuarantineImpl {
        AuthQuarantineImpl::new(failed_attempt_limit, new_cache(), new_lru_ttl_set(ttl))
    }

    fn new_cache() -> Cache<String, u32> {
        Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(NoOpCacheUsageTracker {})),
        )
    }

    fn new_lru_ttl_set(ttl: Duration) -> LruTtlSet<String> {
        LruTtlSet::new(
            NonZeroUsize::new(1).unwrap(),
            ttl,
            Arc::new(Box::new(NoOpCacheUsageTracker {})),
        )
    }
}
