use std::fs;
use std::sync::Arc;

use anyhow::anyhow;
use etcd_client::{Certificate, Identity, TlsOptions};
use etcd_dynamic_state::etcd_api::{EtcdClient, VarPathSpec};
use etcd_dynamic_state::parameter_storage::EtcdParameterStorage;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

use crate::server::configuration::{EtcdConfiguration, EtcdDataFormat};
use crate::server::service::cache::{Cache, CacheUsageTracker};
use crate::server::service::user::UserData;
use crate::server::storage::Storage;

const PATH_DELIMITER: char = '/';

pub struct EtcdStorage {
    inner: Arc<Mutex<EtcdParameterStorage>>,
    cache: Cache<String, (u32, UserData)>,
    path: String,
    data_format: EtcdDataFormat,
}

impl
    TryFrom<(
        &EtcdConfiguration,
        &Runtime,
        Arc<Box<dyn CacheUsageTracker + Send + Sync>>,
    )> for EtcdStorage
{
    type Error = anyhow::Error;

    fn try_from(
        value: (
            &EtcdConfiguration,
            &Runtime,
            Arc<Box<dyn CacheUsageTracker + Send + Sync>>,
        ),
    ) -> Result<Self, Self::Error> {
        let configuration = value.0;
        let runtime = value.1;
        let cache_usage_tracker = value.2;

        let lease_timeout = i64::try_from(configuration.lease_timeout.as_millis())
            .map_err(|e| anyhow!("Invalid lease_timeout: {}", e))?;
        if lease_timeout <= 0 {
            return Err(anyhow!("Invalid lease_timeout: zero or negative"));
        }
        let connect_timeout = u64::try_from(configuration.connect_timeout.as_millis())
            .map_err(|e| anyhow!("Invalid connect_timeout: {}", e))?;
        if connect_timeout == 0 {
            return Err(anyhow!("Invalid connect_timeout: zero"));
        }
        let str_urls: Vec<&str> = configuration.urls.iter().map(AsRef::as_ref).collect();
        let urls = str_urls.as_slice();
        let mut path = configuration.path.clone();
        let path = if path.ends_with(PATH_DELIMITER) {
            path
        } else {
            path.push(PATH_DELIMITER);
            path
        };
        let credentials = configuration
            .credentials
            .as_ref()
            .map(|e| (e.username.as_str(), e.password.as_str()));
        let tls = match &configuration.tls {
            None => None,
            Some(tls_conf) => {
                let mut tls = TlsOptions::default();
                tls = match &tls_conf.server_certificate {
                    Some(path) => {
                        let cert = fs::read(path)?;
                        tls.ca_certificate(Certificate::from_pem(cert))
                    }
                    None => tls,
                };
                tls = match &tls_conf.identity {
                    Some(identity) => {
                        let cert = fs::read(&identity.certificate)?;
                        let key = fs::read(&identity.certificate_key)?;
                        let identity = Identity::from_pem(cert, key);

                        tls.identity(identity)
                    }
                    None => tls,
                };
                Some(tls)
            }
        };
        let client = EtcdClient::new_with_tls(
            urls,
            &credentials,
            path.as_str(),
            lease_timeout,
            connect_timeout,
            tls,
        );

        let client = runtime.block_on(client)?;

        let mut parameter_storage = EtcdParameterStorage::with_client(client);
        parameter_storage.run(runtime)?;
        parameter_storage.order_data_update(VarPathSpec::Prefix(configuration.path.clone()))?;

        let cache = Cache::new(configuration.cache.size, cache_usage_tracker);

        Ok(EtcdStorage {
            inner: Arc::new(Mutex::new(parameter_storage)),
            cache,
            path,
            data_format: configuration.data_format.clone(),
        })
    }
}

impl Storage<UserData> for EtcdStorage {
    fn get(&self, key: &str) -> anyhow::Result<Option<UserData>> {
        let key = format!("{}{}", self.path, key);
        match self.inner.lock().get_data(key.as_str())? {
            None => Ok(None),
            Some((checksum, data)) => {
                let cached = self.cache.get(&key);
                match cached {
                    Some((cache_checksum, cache_data)) if checksum == cache_checksum => {
                        Ok(Some(cache_data))
                    }
                    _ => {
                        let raw_data = String::from_utf8_lossy(&data);
                        let user_data: UserData = match self.data_format {
                            EtcdDataFormat::Json => serde_json::from_str(raw_data.as_ref())?,
                            EtcdDataFormat::Yaml => serde_yaml::from_str(raw_data.as_ref())?,
                        };
                        self.cache.push(key, (checksum, user_data.clone()));
                        Ok(Some(user_data))
                    }
                }
            }
        }
    }
}
