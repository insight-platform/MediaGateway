use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use actix_web::Error;
use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::server::service::user::UserService;

pub async fn basic_auth_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let user_service = req.app_data::<Data<UserService>>();
    if user_service.is_none() {
        return Err((actix_web::error::ErrorInternalServerError(""), req));
    }

    let password = credentials.password();
    if password.is_none()
        || !user_service
            .unwrap()
            .is_valid(credentials.user_id(), password.unwrap())
    {
        return Err((actix_web::error::ErrorUnauthorized(""), req));
    }
    Ok(req)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web_httpauth::headers::authorization::Basic;

    use super::*;

    const ID: &str = "id";
    const PASSWORD: &str = "password";

    #[actix_web::test]
    async fn basic_auth_no_user_service() {
        let service_request = test::TestRequest::default().to_srv_request();

        let result = basic_auth_validator(
            service_request,
            BasicAuth::from(Basic::new("id", Some("password"))),
        )
        .await;
        check_error(result, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn basic_auth_no_password() {
        basic_auth_error(
            HashMap::from([(ID.to_string(), PASSWORD.to_string())]),
            BasicAuth::from(Basic::new(ID, None::<String>)),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }

    #[actix_web::test]
    async fn basic_auth_invalid_credentials() {
        basic_auth_error(
            HashMap::from([(ID.to_string(), PASSWORD.to_string())]),
            BasicAuth::from(Basic::new(ID, Some("_"))),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }
    #[actix_web::test]
    async fn basic_auth_success() {
        let result = get_result(
            HashMap::from([(ID.to_string(), PASSWORD.to_string())]),
            BasicAuth::from(Basic::new(ID, Some(PASSWORD))),
        )
        .await;
        assert!(result.is_ok());
    }

    async fn basic_auth_error(
        users: HashMap<String, String>,
        basic_auth: BasicAuth,
        expected_status_code: StatusCode,
    ) {
        let result = get_result(users, basic_auth).await;
        check_error(result, expected_status_code);
    }

    async fn get_result(
        users: HashMap<String, String>,
        basic_auth: BasicAuth,
    ) -> Result<ServiceRequest, (Error, ServiceRequest)> {
        let user_service = Data::new(UserService::new(users));
        let service_request = test::TestRequest::default()
            .app_data(user_service.clone())
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
}
