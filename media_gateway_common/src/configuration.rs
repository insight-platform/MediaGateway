//! Models for media gateway client and server configurations.
//!
//! The module provides [`Credentials`].
use core::fmt;

use serde::{Deserialize, Serialize};

/// Credentials for basic authentication.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Credentials {
    /// An id (user's name)
    pub username: String,
    /// A password
    pub password: String,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}

impl Clone for Credentials {
    fn clone(&self) -> Self {
        Credentials {
            username: self.username.clone(),
            password: self.password.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.username.clone_from(&source.username);
        self.password.clone_from(&source.password);
    }
}

/// Client TLS settings
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientTlsConfiguration {
    /// A path to a self-signed PEM encoded server certificate or PEM encoded CA certificate
    pub server_certificate: Option<String>,
    /// The identity to be used for client certificate authentication.
    pub identity: Option<Identity>,
}

/// The identity to be used for client certificate authentication.
#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    /// A path to a chain of PEM encoded X509 certificates, with the leaf certificate first
    pub certificate: String,
    /// A path to a PEM encoded private key
    pub certificate_key: String,
}

/// Statistics settings. At least one of frame_period and timestamp_period should be specified.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsConfiguration {
    /// Statistics based on frame period
    pub frame_period: Option<i64>,
    /// Statistics based on timestamp period
    pub timestamp_period: Option<i64>,
    /// A size of a history to be stored
    pub history_size: usize,
}
