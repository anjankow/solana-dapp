// Validates the auth token.
// Extracts user from the auth token.

use std::{collections::HashSet, str::FromStr, sync::Arc};

use jwt_simple::{
    common::VerificationOptions,
    prelude::{HS256Key, MACLike},
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

pub const USER_PUBKEY_HEADER: &str = "user-pubkey";

#[derive(Clone, Debug)]
pub struct AuthMiddlewareConfig {
    auth_secret: Arc<HS256Key>,
    validity: jwt_simple::prelude::Duration,
    time_tolerance: jwt_simple::prelude::Duration,
    allowed_issuers: HashSet<String>,
    token_type: String,
}

impl AuthMiddlewareConfig {
    pub fn new(
        auth_secret: Vec<u8>,
        allowed_issuers: Vec<String>,
        token_validity_sec: u64,
        time_tolerance_sec: u64,
        token_type: String,
    ) -> Self {
        let allowed_issuers_set: HashSet<String> = allowed_issuers.into_iter().collect();

        Self {
            auth_secret: Arc::new(HS256Key::from_bytes(&auth_secret)),
            validity: jwt_simple::prelude::Duration::from_secs(token_validity_sec),
            time_tolerance: jwt_simple::prelude::Duration::from_secs(time_tolerance_sec),
            allowed_issuers: allowed_issuers_set,
            token_type,
        }
    }

    fn new_with_default_values(auth_secret: Vec<u8>, allowed_issuers: Vec<String>) -> Self {
        Self::new(
            auth_secret,
            allowed_issuers,
            7200,
            300,
            "Bearer".to_string(),
        )
    }
}

use bytes::Bytes;
use futures_util::future::BoxFuture;
use http::{HeaderValue, Request, Response, StatusCode};
use http_body_util::Full;
use tower_http::auth::AsyncAuthorizeRequest;

#[derive(Clone)]
struct AppAuth {
    cfg: AuthMiddlewareConfig,
}

impl<B> AsyncAuthorizeRequest<B> for AppAuth
where
    B: Send + Sync + 'static,
{
    type RequestBody = B;
    type ResponseBody = Full<Bytes>;
    type Future = BoxFuture<'static, Result<Request<B>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<B>) -> Self::Future {
        let cfg = self.cfg.clone();
        Box::pin(async move {
            let res = validate_auth_token(cfg.to_owned(), &mut request);
            if let Err(err) = res {
                let status = match err {
                    AuthError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
                    _ => StatusCode::UNAUTHORIZED,
                };
                let resp = http::Response::builder()
                    .status(status)
                    .body(Full::<Bytes>::new(err.to_string().into()))
                    .expect("Failed to build an error http response");
                Err(resp)
            } else {
                Ok(request)
            }
        })
    }
}

fn validate_auth_token<B>(
    cfg: AuthMiddlewareConfig,
    request: &mut Request<B>,
) -> Result<(), AuthError>
where
    B: Send + Sync + 'static,
{
    let headers = request.headers_mut();
    let token = headers.get(http::header::AUTHORIZATION);
    if token.is_none() {
        return Err(AuthError::MissingToken);
    }
    let token = token
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix(&cfg.token_type))
        .map(|token| token.trim());
    if token.is_none() {
        return Err(AuthError::InvalidToken);
    }
    let token = token.unwrap();

    let mut options = VerificationOptions::default();
    options.allowed_issuers = Some(cfg.allowed_issuers.clone());
    options.time_tolerance = Some(cfg.time_tolerance);
    options.max_validity = Some(cfg.validity);

    #[derive(Serialize, Deserialize)]
    struct UserClaims {
        pubkey: String,
    }

    let claims = cfg
        .auth_secret
        .verify_token::<UserClaims>(token, Some(options));
    if let Err(_) = claims {
        return Err(AuthError::InvalidToken);
    }
    let claims = claims.unwrap();
    let pubkey = claims.custom.pubkey;

    let pubkey = Pubkey::from_str(&pubkey).map_err(|_| AuthError::InvalidToken)?;

    // Remove all existing headers that might define 'user-pubkey'
    let _ = headers.remove(USER_PUBKEY_HEADER);
    let pubkey_value =
        HeaderValue::from_str(&pubkey.to_string()).map_err(|_| AuthError::InternalError)?;
    let _ = headers.append(USER_PUBKEY_HEADER, pubkey_value);

    Ok(())
}

use thiserror::Error;
#[derive(Error, Debug)]
enum AuthError {
    #[error("Invalid auth token")]
    InvalidToken,
    #[error("Missing auth token")]
    MissingToken,
    #[error("Authorization failed")]
    InternalError,
}

#[cfg(test)]
mod tests {
    use super::{AppAuth, AuthError, USER_PUBKEY_HEADER};
    use http::{Request, Response, StatusCode};
    use http_body_util::{BodyExt, Full};
    use rstest::rstest;
    use tower_http::auth::AsyncRequireAuthorizationLayer;

    pub async fn test_handler(
        req: Request<http_body_util::Full<bytes::Bytes>>,
    ) -> Result<Response<http_body_util::Full<bytes::Bytes>>, tower::BoxError> {
        let user_header = req
            .headers()
            .get(USER_PUBKEY_HEADER)
            .and_then(|header_value| header_value.to_str().ok())
            .map(|user_header| user_header.to_string());

        let mut res = http::Response::builder();
        let mut body = bytes::Bytes::default();
        if let Some(pubkey) = user_header {
            body = bytes::Bytes::from(pubkey);
        } else {
            // this should never happen - handler won't be called if auth is invalid
            res = res.status(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(res.body(http_body_util::Full::new(body)).unwrap())
    }

    #[tokio::test]
    async fn test_auth_middleware_authorized() {
        let cfg = super::AuthMiddlewareConfig::new_with_default_values(
            "qwertyuiopasdfghjklzxcvbnm123456".as_bytes().to_vec(),
            vec!["test".to_string()],
        );
        let mut service = tower::ServiceBuilder::new()
            .layer(AsyncRequireAuthorizationLayer::new(AppAuth { cfg }))
            .service_fn(test_handler);

        let req = http::Request::builder()
            .method(http::Method::GET)
            .uri("/")
            .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJ0ZXN0IiwiaWF0IjoxNzMwNzM1OTU3LCJleHAiOjI3MDg5NTY3NjQsImF1ZCI6Ind3dy5leGFtcGxlLmNvbSIsInN1YiI6InVzZXIiLCJwdWJrZXkiOiJBUlkzWURLY1RtYTZKTUw5VTRqdXVQdTZTZWRpUXVGWTJEaW9LaXFwa216cCJ9.wD-TfSyZ2k2qcYFjFNRbzOH43ygm3bBA4euoQcWKeJo")
            .body(Full::new(bytes::Bytes::default()))
            .unwrap();

        let res = tower::Service::call(&mut service, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let pubkey = String::from_utf8(
            res.body()
                .to_owned()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        let expected_pubkey = "ARY3YDKcTma6JML9U4juuPu6SediQuFY2DioKiqpkmzp";
        assert_eq!(pubkey, expected_pubkey);

        // println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_pubkey() {
        let cfg = super::AuthMiddlewareConfig::new_with_default_values(
            "qwertyuiopasdfghjklzxcvbnm123456".as_bytes().to_vec(),
            vec!["test".to_string()],
        );
        let mut service = tower::ServiceBuilder::new()
            .layer(AsyncRequireAuthorizationLayer::new(AppAuth { cfg }))
            .service_fn(test_handler);

        let req = http::Request::builder()
            .method(http::Method::GET)
            .uri("/")
            .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJPbmxpbmUgSldUIEJ1aWxkZXIiLCJpYXQiOjE3MzA3NDAyMjQsImV4cCI6MjcwODk2MTAyOSwiYXVkIjoid3d3LmV4YW1wbGUuY29tIiwic3ViIjoianJvY2tldEBleGFtcGxlLmNvbSIsInB1YmtleSI6IkFSWTNZREtjVG1hNkptTUw5VTRqdXVQdTZTZWRpUXVGWTJEaW9LaXFwa216cCJ9.2PJ7mcRKC88mmuX6RybmIAglf7aQ659lhNOF8LeO1oc")
            .body(Full::new(bytes::Bytes::default()))
            .unwrap();

        let res = tower::Service::call(&mut service, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let error_msg = String::from_utf8(
            res.body()
                .to_owned()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(error_msg, AuthError::InvalidToken.to_string());

        // println!("{:?}", res);
    }

    #[rstest]
    #[case("")]
    #[case("Bearer")]
    #[case("APIKey")]
    #[case("Bearer invalid")]
    /* expired */
    #[case("Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJ0ZXN0IiwiaWF0IjoxNzMwNzQwNDA0LCJleHAiOjE3MzA3NDA0MDQsImF1ZCI6Ind3dy5leGFtcGxlLmNvbSIsInN1YiI6Impyb2NrZXRAZXhhbXBsZS5jb20iLCJwdWJrZXkiOiJBUlkzWURLY1RtYTZKbU1MOVU0anV1UHU2U2VkaVF1RlkyRGlvS2lxcGttenAifQ.77s4gLgwBL5qXxGn6je695XS2EhXC8rRtCZvpgyoHgo")]
    #[tokio::test]
    async fn test_auth_middleware_invalid_token(#[case] input: &str) {
        let cfg = super::AuthMiddlewareConfig::new_with_default_values(
            "qwertyuiopasdfghjklzxcvbnm123456".as_bytes().to_vec(),
            vec!["test".to_string()],
        );
        let mut service = tower::ServiceBuilder::new()
            .layer(AsyncRequireAuthorizationLayer::new(AppAuth { cfg }))
            .service_fn(test_handler);

        let req = http::Request::builder()
            .method(http::Method::GET)
            .uri("/")
            .header("Authorization", input)
            .body(Full::new(bytes::Bytes::default()))
            .unwrap();

        let res = tower::Service::call(&mut service, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let error_msg = String::from_utf8(
            res.body()
                .to_owned()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(error_msg, AuthError::InvalidToken.to_string());

        // println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_token() {
        let cfg = super::AuthMiddlewareConfig::new_with_default_values(
            "qwertyuiopasdfghjklzxcvbnm123456".as_bytes().to_vec(),
            vec!["test".to_string()],
        );
        let mut service = tower::ServiceBuilder::new()
            .layer(AsyncRequireAuthorizationLayer::new(AppAuth { cfg }))
            .service_fn(test_handler);

        let req = http::Request::builder()
            .method(http::Method::GET)
            .uri("/")
            .body(Full::new(bytes::Bytes::default()))
            .unwrap();

        let res = tower::Service::call(&mut service, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let error_msg = String::from_utf8(
            res.body()
                .to_owned()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert_eq!(error_msg, AuthError::MissingToken.to_string());

        // println!("{:?}", res);
    }
}
