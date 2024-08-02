use mockall::automock;

pub mod argon2;

#[automock]
pub trait PasswordService {
    fn verify(&self, password: &str, password_hash: &str) -> anyhow::Result<bool>;
}
