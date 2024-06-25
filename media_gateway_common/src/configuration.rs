use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BasicUser {
    pub id: String,
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
