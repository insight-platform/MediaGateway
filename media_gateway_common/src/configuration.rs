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
