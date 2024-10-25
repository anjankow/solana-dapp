// Extracts host from the request URL.
// A safer version of built-in Host extractor,
// which resolves the Host by primarily checking
// the headers, and as the documentation says:
// Note that user agents can set X-Forwarded-Host
// and Host headers to arbitrary values so make
// sure to validate them to avoid security issues.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

pub struct ExtractHostname(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractHostname
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(host) = parts.uri.host() {
            Ok(ExtractHostname(host.to_string()))
        } else {
            Err((StatusCode::BAD_REQUEST, "Invalid request URI"))
        }
    }
}
