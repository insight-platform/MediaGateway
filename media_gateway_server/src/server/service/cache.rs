use std::borrow::Borrow;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::ops::Add;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use anyhow::bail;
use lru::LruCache;
use mockall::automock;
use parking_lot::Mutex;
use tokio::runtime::Runtime;
use tokio::signal::{ctrl_c, unix};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::time::sleep;

use crate::server::configuration::CacheUsage;

pub struct Cache<K, V> {
    inner: Arc<Mutex<LruCache<K, V>>>,
    cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Sync + Send>>,
}

impl<K: Hash + Eq + Clone, V: Clone> Cache<K, V> {
    pub fn new(
        cache_size: NonZeroUsize,
        cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Sync + Send>>,
    ) -> Self {
        Cache {
            inner: Arc::new(Mutex::new(LruCache::new(cache_size))),
            cache_usage_tracker,
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut cache = self.inner.lock();
        cache.get(key).cloned()
    }

    pub fn push(&self, key: K, value: V) -> Option<(K, V)> {
        let mut cache = self.inner.lock();
        let pushed_key = key.clone();
        let result = cache.push(key, value);
        if let Some((cached_key, _)) = result.as_ref() {
            if cached_key != &pushed_key {
                self.cache_usage_tracker.register_evicted();
            }
        }
        result
    }

    pub fn pop<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut cache = self.inner.lock();
        cache.pop(key)
    }
}

pub struct LruTtlSet<T> {
    inner: Arc<Mutex<LruCache<T, Instant>>>,
    ttl: Duration,
    size: NonZeroUsize,
    cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Sync + Send>>,
}

impl<T: Hash + Eq + Clone> LruTtlSet<T> {
    pub fn new(
        size: NonZeroUsize,
        ttl: Duration,
        cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Sync + Send>>,
    ) -> Self {
        LruTtlSet {
            inner: Arc::new(Mutex::new(LruCache::new(size))),
            ttl,
            size,
            cache_usage_tracker,
        }
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut cache = self.inner.lock();
        match cache.get(key) {
            Some(expiration) => {
                if *expiration <= Instant::now() {
                    cache.pop(key);
                    false
                } else {
                    true
                }
            }
            None => false,
        }
    }

    pub fn add(&self, key: T) {
        let mut cache = self.inner.lock();
        let now = Instant::now();
        if cache.len() == self.size.get() {
            let keys = cache
                .iter()
                .filter(|e| *e.1 <= now)
                .map(|e| e.0.clone())
                .collect::<Vec<T>>();
            for key in keys {
                cache.pop(&key);
            }
        }
        let pushed_key = key.clone();
        let result = cache.push(key, now.add(self.ttl));
        if let Some((cached_key, _)) = result.as_ref() {
            if cached_key != &pushed_key {
                self.cache_usage_tracker.register_evicted();
            }
        }
    }
}

pub(crate) struct CacheStatistics {
    evicted: u64,
}

impl CacheStatistics {
    pub fn new() -> Self {
        CacheStatistics { evicted: 0 }
    }

    pub fn register_evicted(&mut self) {
        match self.evicted.checked_add(1) {
            Some(result) => self.evicted = result,
            None => {
                log::warn!("Evicted statistics overflow, resetting");
                self.evicted = 0;
            }
        }
    }

    pub fn reset(&mut self) -> Self {
        let evicted = self.evicted;
        self.evicted = 0;
        CacheStatistics { evicted }
    }
}

#[automock]
pub trait CacheUsageTracker {
    fn name(&self) -> &str;

    fn register_evicted(&self);

    fn reset(&self) -> CacheStatistics;
}

pub struct NoOpCacheUsageTracker {}

impl CacheUsageTracker for NoOpCacheUsageTracker {
    fn name(&self) -> &str {
        "NoOp"
    }

    fn register_evicted(&self) {
        // do nothing
    }

    fn reset(&self) -> CacheStatistics {
        CacheStatistics::new()
    }
}

pub struct CacheUsageTrackerImpl {
    statistics: Arc<Mutex<CacheStatistics>>,
    name: String,
}

impl CacheUsageTrackerImpl {
    pub fn new(name: String) -> Self {
        CacheUsageTrackerImpl {
            statistics: Arc::new(Mutex::new(CacheStatistics::new())),
            name,
        }
    }
}

impl CacheUsageTracker for CacheUsageTrackerImpl {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn register_evicted(&self) {
        let mut statistics = self.statistics.lock();
        statistics.register_evicted()
    }

    fn reset(&self) -> CacheStatistics {
        let mut statistics = self.statistics.lock();
        statistics.reset()
    }
}

pub trait CacheUsageService {
    async fn run(&self, shutdown_signal: Receiver<()>) -> anyhow::Result<()>;
}

pub struct EvictedThresholdWarningCacheUsageService {
    period: Duration,
    threshold_per_second: f64,
    cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Send + Sync>>,
    started: Arc<OnceLock<()>>,
}

impl EvictedThresholdWarningCacheUsageService {
    pub fn new(
        period: Duration,
        threshold: u32,
        cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Send + Sync>>,
    ) -> Self {
        let threshold_per_second = (threshold as f64) / period.as_secs_f64();
        EvictedThresholdWarningCacheUsageService {
            period,
            threshold_per_second,
            cache_usage_tracker,
            started: Arc::new(OnceLock::new()),
        }
    }
}

impl CacheUsageService for EvictedThresholdWarningCacheUsageService {
    async fn run(&self, mut shutdown_signal: Receiver<()>) -> anyhow::Result<()> {
        let started_result = self.started.set(());
        if started_result.is_err() {
            bail!(
                "Cache usage service {} has already been started.",
                self.cache_usage_tracker.name()
            )
        }
        log::info!(
            "Cache usage service {} is started",
            self.cache_usage_tracker.name()
        );
        let mut start = Instant::now();
        loop {
            tokio::select! {
                _ = &mut shutdown_signal => {
                    log::info!("Cache usage service {} is being stopped", self.cache_usage_tracker.name());
                    break;
                },
                _ = sleep(self.period) => {
                    let statistics = self.cache_usage_tracker.reset();
                    let current = Instant::now();
                    let duration = current
                        .checked_duration_since(start)
                        .map(|e| e.as_secs_f64());
                    if duration.is_some()
                        && (statistics.evicted as f64) / duration.unwrap() > self.threshold_per_second {
                        log::warn!(
                            "Evicted entities threshold is exceeded for {}: {} per {:.3} seconds",
                            self.cache_usage_tracker.name(),
                            statistics.evicted,
                            duration.unwrap()
                        )
                    }
                    start = current;
                }
            }
        }
        log::info!(
            "Cache usage service {} is stopped",
            self.cache_usage_tracker.name()
        );
        Ok(())
    }
}
pub struct CacheUsageFactory {}

impl CacheUsageFactory {
    pub fn from(
        value: Option<&CacheUsage>,
        name: String,
        runtime: &Runtime,
    ) -> Arc<Box<dyn CacheUsageTracker + Send + Sync>> {
        match value {
            None => Arc::new(Box::new(NoOpCacheUsageTracker {})),
            Some(config) => {
                let cache_usage_tracker: Arc<Box<dyn CacheUsageTracker + Send + Sync>> =
                    Arc::new(Box::new(CacheUsageTrackerImpl::new(name.clone())));
                let (tx, rx) = oneshot::channel();
                let cache_usage_service = EvictedThresholdWarningCacheUsageService::new(
                    config.period,
                    config.evicted_threshold,
                    cache_usage_tracker.clone(),
                );
                let run_name = name.clone();
                runtime.spawn(async move {
                    cache_usage_service.run(rx).await.unwrap_or_else(|_| {
                        panic!("Error while running cache usage service {}", run_name)
                    })
                });
                runtime.spawn(async move {
                    let mut interrupt_signal = unix::signal(unix::SignalKind::interrupt()).unwrap();
                    let mut shutdown_signal = unix::signal(unix::SignalKind::terminate()).unwrap();
                    let mut quit_signal = unix::signal(unix::SignalKind::quit()).unwrap();
                    tokio::select! {
                        _ = ctrl_c() => {},
                        _ = interrupt_signal.recv() => {}
                        _ = shutdown_signal.recv() => {}
                        _ = quit_signal.recv() => {}
                    }
                    tx.send(()).unwrap_or_else(|_| {
                        panic!("Error while stopping cache usage service {}", name)
                    });
                });
                cache_usage_tracker
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::thread::sleep;
    use std::time::Duration;

    use crate::server::service::cache::{Cache, LruTtlSet, MockCacheUsageTracker};

    const KEY: u32 = 1;
    const VALUE: &str = "value";

    #[test]
    pub fn cache_get_no_entry() {
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        let result = cache.get(&KEY);

        assert_eq!(result, None);
    }

    #[test]
    pub fn cache_get_existing_entry() {
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        cache.push(KEY, VALUE);

        let result = cache.get(&KEY);

        assert_eq!(result, Some(VALUE));
    }

    #[test]
    pub fn cache_push_no_entries() {
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        let result = cache.push(KEY, VALUE);

        assert_eq!(result, None);
    }

    #[test]
    pub fn cache_push_same_entity() {
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        let result = cache.push(KEY, VALUE);

        assert_eq!(result, None);

        let result = cache.push(KEY, "another value");

        assert_eq!(result, Some((KEY, VALUE)));
    }

    #[test]
    pub fn cache_push_evicted_entity() {
        let mut cache_usage_tracker = MockCacheUsageTracker::new();
        cache_usage_tracker
            .expect_register_evicted()
            .return_const(())
            .once();
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(cache_usage_tracker)),
        );

        let result = cache.push(KEY, VALUE);

        assert_eq!(result, None);

        let result = cache.push(2, "another value");

        assert_eq!(result, Some((KEY, VALUE)));
    }

    #[test]
    pub fn cache_pop_no_entry() {
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        let result = cache.pop(&KEY);

        assert_eq!(result, None);
    }
    #[test]
    pub fn cache_pop_existing_entry() {
        let cache: Cache<u32, &str> = Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        cache.push(KEY, VALUE);

        let result = cache.pop(&KEY);

        assert_eq!(result, Some(VALUE));
    }

    #[test]
    pub fn lru_ttl_set_contains_no_entries() {
        let set: LruTtlSet<i32> = LruTtlSet::new(
            NonZeroUsize::new(10).unwrap(),
            Duration::from_secs(1),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        assert!(!set.contains(&1));
    }

    #[test]
    pub fn lru_ttl_set_add() {
        let val = 1;
        let set = LruTtlSet::new(
            NonZeroUsize::new(10).unwrap(),
            Duration::from_secs(1),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        set.add(&val);

        assert!(set.contains(&val));
    }

    #[test]
    pub fn lru_ttl_set_contains_another_entry() {
        let set = LruTtlSet::new(
            NonZeroUsize::new(10).unwrap(),
            Duration::from_secs(1),
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        set.add(&1);

        assert!(!set.contains(&2));
    }

    #[test]
    pub fn lru_ttl_set_contains_expired_entry() {
        let val = 1;
        let duration = Duration::from_millis(10);
        let set = LruTtlSet::new(
            NonZeroUsize::new(10).unwrap(),
            duration,
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        set.add(&val);
        assert!(set.contains(&val));

        sleep(duration);

        assert!(!set.contains(&val));
    }

    #[test]
    pub fn lru_ttl_add_full_capacity_evicted_entity() {
        let existing_value = 1;
        let new_value = 2;
        let mut cache_usage_tracker = MockCacheUsageTracker::new();
        cache_usage_tracker
            .expect_register_evicted()
            .return_const(())
            .once();
        let set = LruTtlSet::new(
            NonZeroUsize::new(1).unwrap(),
            Duration::from_millis(1000),
            Arc::new(Box::new(cache_usage_tracker)),
        );

        set.add(&existing_value);
        set.add(&new_value);

        assert!(!set.contains(&existing_value));
        assert!(set.contains(&new_value));
    }

    #[test]
    pub fn lru_ttl_add_full_capacity_expired_entity() {
        let duration = Duration::from_millis(10);
        let existing_value = 1;
        let expired_value = 2;
        let new_value = 3;
        let set = LruTtlSet::new(
            NonZeroUsize::new(2).unwrap(),
            duration,
            Arc::new(Box::new(MockCacheUsageTracker::new())),
        );

        set.add(&expired_value);

        sleep(duration);

        set.add(&existing_value);
        set.add(&new_value);

        assert!(!set.contains(&expired_value));
        assert!(set.contains(&existing_value));
        assert!(set.contains(&new_value));
    }
}
