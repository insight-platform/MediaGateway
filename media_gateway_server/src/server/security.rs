use crate::server::service::cache::Cache;
use crate::server::service::crypto::PasswordService;
use crate::server::service::user::UserService;
use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use actix_web_httpauth::extractors::basic::BasicAuth;
use anyhow::anyhow;
use log::error;
use media_gateway_common::configuration::Credentials;

fn to_credentials(value: &BasicAuth) -> Result<Credentials, anyhow::Error> {
    if let Some(password) = value.password() {
        Ok(Credentials {
            username: value.user_id().to_string(),
            password: password.to_string(),
        })
    } else {
        Err(anyhow!("Empty password"))
    }
}

pub struct BasicAuthCheckResult {
    valid: bool,
    password_hash: String,
}

impl BasicAuthCheckResult {
    pub fn valid(password_hash: String) -> Self {
        BasicAuthCheckResult {
            valid: true,
            password_hash,
        }
    }

    pub fn invalid(password_hash: String) -> Self {
        BasicAuthCheckResult {
            valid: false,
            password_hash,
        }
    }
}

impl Clone for BasicAuthCheckResult {
    fn clone(&self) -> Self {
        BasicAuthCheckResult {
            valid: self.valid,
            password_hash: self.password_hash.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.valid.clone_from(&source.valid);
        self.password_hash.clone_from(&source.password_hash);
    }
}

pub async fn basic_auth_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let user_service = req.app_data::<Data<UserService>>();
    if user_service.is_none() {
        error!("No user service");
        return Err((actix_web::error::ErrorInternalServerError(""), req));
    }
    let user_service = user_service.unwrap();

    let password_service = req.app_data::<Data<Box<dyn PasswordService + Sync + Send>>>();
    if password_service.is_none() {
        error!("No password service");
        return Err((actix_web::error::ErrorInternalServerError(""), req));
    }
    let password_service = password_service.unwrap();

    let basic_auth_cache = req.app_data::<Data<Cache<Credentials, BasicAuthCheckResult>>>();
    if basic_auth_cache.is_none() {
        error!("No basic auth cache");
        return Err((actix_web::error::ErrorInternalServerError(""), req));
    }
    let basic_auth_cache = basic_auth_cache.unwrap();

    let password = credentials.password();
    if password.is_none() {
        return Err((actix_web::error::ErrorUnauthorized(""), req));
    }

    let credentials = to_credentials(&credentials).unwrap();
    let user_data_result = user_service.get(credentials.username.as_str());
    match user_data_result {
        Err(e) => {
            error!("Error while retrieving user data: {:?}", e);
            Err((actix_web::error::ErrorInternalServerError(""), req))
        }
        Ok(None) => Err((actix_web::error::ErrorUnauthorized(""), req)),
        Ok(Some(user_data)) => {
            let cache_result = basic_auth_cache.get(&credentials);
            match cache_result {
                Some(e) if e.password_hash == user_data.password_hash => {
                    if e.valid {
                        req.extensions_mut().insert(user_data);
                        Ok(req)
                    } else {
                        Err((actix_web::error::ErrorUnauthorized(""), req))
                    }
                }
                _ => {
                    match password_service.verify(
                        credentials.password.as_str(),
                        user_data.password_hash.as_str(),
                    ) {
                        Err(e) => {
                            error!("Error while verifying a user password: {:?}", e);
                            basic_auth_cache.push(
                                credentials,
                                BasicAuthCheckResult::invalid(user_data.password_hash.clone()),
                            );
                            Err((actix_web::error::ErrorUnauthorized(""), req))
                        }
                        Ok(true) => {
                            basic_auth_cache.push(
                                credentials,
                                BasicAuthCheckResult::valid(user_data.password_hash.clone()),
                            );
                            req.extensions_mut().insert(user_data);
                            Ok(req)
                        }
                        Ok(false) => {
                            basic_auth_cache.push(
                                credentials,
                                BasicAuthCheckResult::invalid(user_data.password_hash.clone()),
                            );
                            Err((actix_web::error::ErrorUnauthorized(""), req))
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web_httpauth::headers::authorization::Basic;
    use anyhow::anyhow;
    use mockall::predicate::eq;

    use crate::server::service::cache::NoOpCacheUsageTracker;
    use crate::server::service::crypto::MockPasswordService;
    use crate::server::service::user::UserData;
    use crate::server::storage::{MockStorage, Storage};

    use super::*;

    const ID: &str = "id";
    const PASSWORD: &str = "password";
    const PASSWORD_HASH: &str = "password_hash";

    #[actix_web::test]
    async fn basic_auth_no_user_service() {
        let service_request = test::TestRequest::default().to_srv_request();

        let result = basic_auth_validator(
            service_request,
            BasicAuth::from(Basic::new(ID, Some(PASSWORD))),
        )
        .await;
        check_error(result, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn basic_no_password_service() {
        let user_service = Data::new(UserService::new(storage_to_box(MockStorage::new())));
        let service_request = test::TestRequest::default()
            .app_data(user_service.clone())
            .to_srv_request();

        let result = basic_auth_validator(
            service_request,
            BasicAuth::from(Basic::new(ID, Some(PASSWORD))),
        )
        .await;
        check_error(result, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn basic_no_basic_auth_cache() {
        let user_service = Data::new(UserService::new(storage_to_box(MockStorage::new())));
        let password_service = Data::new(password_service_to_box(MockPasswordService::new()));
        let service_request = test::TestRequest::default()
            .app_data(user_service.clone())
            .app_data(password_service.clone())
            .to_srv_request();

        let result = basic_auth_validator(
            service_request,
            BasicAuth::from(Basic::new(ID, Some(PASSWORD))),
        )
        .await;
        check_error(result, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn basic_auth_no_password() {
        basic_auth_error(
            storage_to_box(MockStorage::new()),
            password_service_to_box(MockPasswordService::new()),
            new_cache(),
            BasicAuth::from(Basic::new(ID, None::<String>)),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }

    #[actix_web::test]
    async fn basic_auth_user_service_error() {
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(ID))
            .times(1)
            .returning(|_x| Err(anyhow!("error")));

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(MockPasswordService::new()),
            new_cache(),
            BasicAuth::from(Basic::new(ID, Some(PASSWORD))),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .await;
    }

    #[actix_web::test]
    async fn basic_auth_user_service_no_user_no_cache_data() {
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(ID))
            .times(1)
            .returning(|_x| Ok(None));

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(MockPasswordService::new()),
            new_cache(),
            BasicAuth::from(Basic::new(ID, Some(PASSWORD))),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }

    #[actix_web::test]
    async fn basic_auth_user_service_no_user_cache_data() {
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(ID))
            .times(1)
            .returning(|_x| Ok(None));
        let cache = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();
        cache.push(
            credentials,
            BasicAuthCheckResult::valid(PASSWORD_HASH.to_string()),
        );

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(MockPasswordService::new()),
            cache.clone(),
            basic_auth,
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }

    #[actix_web::test]
    async fn basic_auth_password_service_error() {
        let mut storage = MockStorage::new();
        storage.expect_get().with(eq(ID)).times(1).returning(|_x| {
            Ok(Some(UserData {
                password_hash: PASSWORD_HASH.to_string(),
                allowed_routing_labels: None,
            }))
        });
        let mut password_service = MockPasswordService::new();
        password_service
            .expect_verify()
            .with(eq(PASSWORD), eq(PASSWORD_HASH))
            .times(1)
            .returning(|_x, _y| Err(anyhow!("error")));
        let cache: Data<Cache<Credentials, BasicAuthCheckResult>> = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(password_service),
            cache.clone(),
            basic_auth,
            StatusCode::UNAUTHORIZED,
        )
        .await;

        let cache_result = cache.get(&credentials);

        assert!(cache_result
            .is_some_and(|e| e.valid == false && e.password_hash == PASSWORD_HASH.to_string()));
    }

    #[actix_web::test]
    async fn basic_auth_invalid_credentials_no_cache_data() {
        let mut storage = MockStorage::new();
        storage.expect_get().with(eq(ID)).times(1).returning(|_x| {
            Ok(Some(UserData {
                password_hash: PASSWORD_HASH.to_string(),
                allowed_routing_labels: None,
            }))
        });
        let mut password_service = MockPasswordService::new();
        password_service
            .expect_verify()
            .with(eq(PASSWORD), eq(PASSWORD_HASH))
            .times(1)
            .returning(|_x, _y| Ok(false));
        let cache: Data<Cache<Credentials, BasicAuthCheckResult>> = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(password_service),
            cache.clone(),
            basic_auth,
            StatusCode::UNAUTHORIZED,
        )
        .await;

        let cache_result = cache.get(&credentials);

        assert!(cache_result
            .is_some_and(|e| e.valid == false && e.password_hash == PASSWORD_HASH.to_string()));
    }

    #[actix_web::test]
    async fn basic_auth_invalid_credentials_cache_data_same_password_hash() {
        let mut storage = MockStorage::new();
        storage.expect_get().with(eq(ID)).times(1).returning(|_x| {
            Ok(Some(UserData {
                password_hash: PASSWORD_HASH.to_string(),
                allowed_routing_labels: None,
            }))
        });
        let password_service = MockPasswordService::new();
        let cache: Data<Cache<Credentials, BasicAuthCheckResult>> = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();
        cache.push(
            credentials.clone(),
            BasicAuthCheckResult::invalid(PASSWORD_HASH.to_string()),
        );

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(password_service),
            cache.clone(),
            basic_auth,
            StatusCode::UNAUTHORIZED,
        )
        .await;

        let cache_result = cache.get(&credentials);

        assert!(cache_result
            .is_some_and(|e| e.valid == false && e.password_hash == PASSWORD_HASH.to_string()));
    }

    #[actix_web::test]
    async fn basic_auth_invalid_credentials_cache_data_old_password_hash() {
        let new_password_hash = "new_password_hash";
        let mut storage = MockStorage::new();
        storage.expect_get().with(eq(ID)).times(1).returning(|_x| {
            Ok(Some(UserData {
                password_hash: new_password_hash.to_string(),
                allowed_routing_labels: None,
            }))
        });
        let mut password_service = MockPasswordService::new();
        password_service
            .expect_verify()
            .with(eq(PASSWORD), eq(new_password_hash))
            .times(1)
            .returning(|_x, _y| Ok(false));
        let cache: Data<Cache<Credentials, BasicAuthCheckResult>> = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();
        cache.push(
            credentials.clone(),
            BasicAuthCheckResult::valid(PASSWORD_HASH.to_string()),
        );

        basic_auth_error(
            storage_to_box(storage),
            password_service_to_box(password_service),
            cache.clone(),
            basic_auth,
            StatusCode::UNAUTHORIZED,
        )
        .await;

        let cache_result = cache.get(&credentials);

        assert!(cache_result
            .is_some_and(|e| e.valid == false && e.password_hash == new_password_hash.to_string()));
    }

    #[actix_web::test]
    async fn basic_auth_success_no_cache_data() {
        let user_data = UserData {
            password_hash: PASSWORD_HASH.to_string(),
            allowed_routing_labels: None,
        };
        let storage_user_data = Some(user_data.clone());
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(ID))
            .return_once(|_x| Ok(storage_user_data));
        let mut password_service = MockPasswordService::new();
        password_service
            .expect_verify()
            .with(eq(PASSWORD), eq(PASSWORD_HASH))
            .times(1)
            .returning(|_x, _y| Ok(true));
        let cache: Data<Cache<Credentials, BasicAuthCheckResult>> = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();

        let result = get_result(
            storage_to_box(storage),
            password_service_to_box(password_service),
            cache.clone(),
            basic_auth,
        )
        .await;
        assert!(result.is_ok());
        assert!(result
            .unwrap()
            .extensions()
            .get::<UserData>()
            .is_some_and(|e| e == &user_data));

        let cache_result = cache.get(&credentials);

        assert!(cache_result
            .is_some_and(|e| e.valid == true && e.password_hash == PASSWORD_HASH.to_string()));
    }

    #[actix_web::test]
    async fn basic_auth_success_cache_data() {
        let user_data = UserData {
            password_hash: PASSWORD_HASH.to_string(),
            allowed_routing_labels: None,
        };
        let storage_user_data = Some(user_data.clone());
        let mut storage = MockStorage::new();
        storage
            .expect_get()
            .with(eq(ID))
            .return_once(|_x| Ok(storage_user_data));
        let password_service = MockPasswordService::new();
        let cache: Data<Cache<Credentials, BasicAuthCheckResult>> = new_cache();
        let basic_auth = BasicAuth::from(Basic::new(ID, Some(PASSWORD)));
        let credentials = to_credentials(&basic_auth).unwrap();
        cache.push(
            credentials.clone(),
            BasicAuthCheckResult::valid(PASSWORD_HASH.to_string()),
        );

        let result = get_result(
            storage_to_box(storage),
            password_service_to_box(password_service),
            cache.clone(),
            basic_auth,
        )
        .await;
        assert!(result.is_ok());
        assert!(result
            .unwrap()
            .extensions()
            .get::<UserData>()
            .is_some_and(|e| e == &user_data));

        let cache_result = cache.get(&credentials);

        assert!(cache_result
            .is_some_and(|e| e.valid == true && e.password_hash == PASSWORD_HASH.to_string()));
    }

    async fn basic_auth_error(
        storage: Box<dyn Storage<UserData> + Send + Sync>,
        password_service: Box<dyn PasswordService + Send + Sync>,
        cache: Data<Cache<Credentials, BasicAuthCheckResult>>,
        basic_auth: BasicAuth,
        expected_status_code: StatusCode,
    ) {
        let result = get_result(storage, password_service, cache, basic_auth).await;
        check_error(result, expected_status_code);
    }

    async fn get_result(
        storage: Box<dyn Storage<UserData> + Send + Sync>,
        password_service: Box<dyn PasswordService + Send + Sync>,
        cache: Data<Cache<Credentials, BasicAuthCheckResult>>,
        basic_auth: BasicAuth,
    ) -> Result<ServiceRequest, (Error, ServiceRequest)> {
        let user_service = Data::new(UserService::new(storage));
        let password_service = Data::new(password_service);
        let service_request = test::TestRequest::default()
            .app_data(user_service.clone())
            .app_data(password_service.clone())
            .app_data(cache.clone())
            .to_srv_request();

        basic_auth_validator(service_request, basic_auth).await
    }

    fn check_error(
        result: Result<ServiceRequest, (Error, ServiceRequest)>,
        error_status_code: StatusCode,
    ) {
        match result {
            Err((e, _)) => {
                let response = e.error_response();
                assert_eq!(response.status(), error_status_code);
            }
            Ok(_) => panic!("Unexpected Ok result"),
        }
    }

    fn storage_to_box(storage: MockStorage<UserData>) -> Box<dyn Storage<UserData> + Send + Sync> {
        Box::new(storage)
    }

    fn password_service_to_box(
        password_service: MockPasswordService,
    ) -> Box<dyn PasswordService + Send + Sync> {
        Box::new(password_service)
    }

    fn new_cache() -> Data<Cache<Credentials, BasicAuthCheckResult>> {
        Data::new(Cache::new(
            NonZeroUsize::new(1).unwrap(),
            Arc::new(Box::new(NoOpCacheUsageTracker {})),
        ))
    }
}
