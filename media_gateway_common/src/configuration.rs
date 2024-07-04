//! Models for media gateway client and server configurations.
//!
//! The module provides [`BasicUser`].
use core::fmt;

use serde::{Deserialize, Serialize};

/// Credentials for basic authentication.
#[derive(Serialize, Deserialize)]
pub struct BasicUser {
    /// An id (user's name)
    pub id: String,
    /// A password
    pub password: String,
}

impl fmt::Debug for BasicUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicUser")
            .field("id", &self.id)
            .field("password", &"***")
            .finish()
    }
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
