use savant_core::message::label_filter::LabelFilterRule;
use serde::{Deserialize, Serialize};

use crate::server::storage::Storage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserData {
    pub password_hash: String,
    pub allowed_routing_labels: Option<LabelFilterRule>,
}

pub struct UserService {
    storage: Box<dyn Storage<UserData> + Sync + Send>,
}

impl UserService {
    pub fn new(storage: Box<dyn Storage<UserData> + Sync + Send>) -> Self {
        UserService { storage }
    }

    pub fn get(&self, user_name: &str) -> anyhow::Result<Option<UserData>> {
        self.storage.as_ref().get(user_name)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use mockall::predicate::eq;
    use savant_core::message::label_filter::LabelFilterRule;

    use crate::server::service::user::{UserData, UserService};
    use crate::server::storage::MockStorage;

    const USERNAME: &str = "username";
    const PASSWORD_HASH: &str = "password_hash";

    #[test]
    fn get_no_user() {
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(USERNAME))
            .times(1)
            .returning(|_x| Ok(None));
        let service = UserService::new(Box::new(storage));

        let result = service.get(USERNAME);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn get_user() {
        let user_data = UserData {
            password_hash: PASSWORD_HASH.to_string(),
            allowed_routing_labels: Some(LabelFilterRule::Set("label".to_string())),
        };
        let storage_user_data = user_data.clone();
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(USERNAME))
            .return_once(move |_x| Ok(Some(storage_user_data)));
        let service = UserService::new(Box::new(storage));

        let result = service.get(USERNAME);

        assert!(result.is_ok());
        assert!(result.unwrap().is_some_and(|e| e == user_data));
    }

    #[test]
    fn get_error() {
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(USERNAME))
            .times(1)
            .returning(|_x| Err(anyhow!("error")));
        let service = UserService::new(Box::new(storage));

        let result = service.get(USERNAME);

        assert!(result.is_err());
    }
}
