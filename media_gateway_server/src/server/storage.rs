use mockall::automock;

pub mod etcd;

#[automock]
pub trait Storage<T> {
    fn get(&self, key: &str) -> anyhow::Result<Option<T>>;
}

pub struct EmptyStorage {}

impl<T> Storage<T> for EmptyStorage {
    fn get(&self, _key: &str) -> anyhow::Result<Option<T>> {
        Ok(None)
    }
}
