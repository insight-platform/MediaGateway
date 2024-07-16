use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::server::service::crypto::PasswordService;

pub struct Argon2PasswordService {}

impl PasswordService for Argon2PasswordService {
    fn verify(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| anyhow!("Error while parsing password hash: {:?}", e))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use crate::server::service::crypto::argon2::Argon2PasswordService;
    use crate::server::service::crypto::PasswordService;

    const PASSWORD: &str = "password";
    const PASSWORD_HASH: &str = "$argon2id$v=19$m=4096,t=3,p=1$c2FsdDEyMzQ1Njc4OTA$sdNXWermhXxauW3W98UYhuaRClvD/ASQeTbXj8Fz68o";
    const ANOTHER_PASSWORD: &str = "passwort";

    #[test]
    pub fn verify_same_password() {
        let service = Argon2PasswordService {};

        let result = service.verify(PASSWORD, PASSWORD_HASH);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    pub fn verify_different_password() {
        let service = Argon2PasswordService {};

        let result = service.verify(ANOTHER_PASSWORD, PASSWORD_HASH);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    pub fn verify_invalid_password_hash() {
        let password_hash = "$6$beUe7YyPznqUo.Mg$IehBgp2Jb89zZPiSx9/pq6VGCTSAbvHAjbPGKv0Z6fq1zu8h2Aj0S83lBYXiGmD0lChY4jIBXKr55GpmTdg81.";

        let service = Argon2PasswordService {};

        let result = service.verify(PASSWORD, password_hash);
        assert!(result.is_err());
    }
}
