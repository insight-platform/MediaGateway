use std::collections::HashMap;

pub struct UserService {
    users: HashMap<String, String>,
}

impl UserService {
    pub fn new(users: HashMap<String, String>) -> Self {
        UserService { users }
    }

    pub fn is_valid(&self, username: &str, password: &str) -> bool {
        self.users.get(username).map(|e| e.as_str()) == Some(password)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::server::service::user::UserService;

    #[test]
    fn is_valid_no_users() {
        let service = UserService::new(HashMap::new());

        assert_eq!(service.is_valid("id", "password"), false);
    }

    #[test]
    fn is_valid_absent_user() {
        let service = UserService::new(HashMap::from([("u1".to_string(), "p1".to_string())]));

        assert_eq!(service.is_valid("u2", "p"), false);
    }

    #[test]
    fn is_valid_invalid_password() {
        let service = UserService::new(HashMap::from([("u1".to_string(), "p1".to_string())]));

        assert_eq!(service.is_valid("u1", "p"), false);
    }

    #[test]
    fn is_valid_valid_user() {
        let service = UserService::new(HashMap::from([
            ("u1".to_string(), "p1".to_string()),
            ("u2".to_string(), "p2".to_string()),
        ]));

        assert_eq!(service.is_valid("u2", "p2"), true);
    }
}
