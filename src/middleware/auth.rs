use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::AppState;

// ---------------------------------------------------------------------------
// Firebase public key cache
// ---------------------------------------------------------------------------

const FIREBASE_CERTS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

/// A thread-safe cache of Firebase public keys (kid → PEM string).
#[derive(Clone, Default)]
pub struct FirebaseKeyCache(pub Arc<RwLock<HashMap<String, String>>>);

impl FirebaseKeyCache {
    pub async fn get_keys(&self, client: &Client) -> anyhow::Result<HashMap<String, String>> {
        // Return cached keys if present.
        {
            let cache = self.0.read().await;
            if !cache.is_empty() {
                return Ok(cache.clone());
            }
        }
        // Fetch fresh keys from Google.
        let keys: HashMap<String, String> = client
            .get(FIREBASE_CERTS_URL)
            .send()
            .await?
            .json()
            .await?;
        let mut cache = self.0.write().await;
        *cache = keys.clone();
        Ok(keys)
    }

    /// Invalidate the cache (called when a kid is not found, so we re-fetch).
    pub async fn invalidate(&self) {
        let mut cache = self.0.write().await;
        cache.clear();
    }
}

// ---------------------------------------------------------------------------
// JWT claims
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct FirebaseClaims {
    /// Subject — the Firebase UID.
    pub sub: String,
    pub email: Option<String>,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
}

// ---------------------------------------------------------------------------
// Extractor: AuthenticatedUser
// ---------------------------------------------------------------------------

/// Injected into handlers that require a verified Firebase token.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub uid: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract Bearer token.
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError::MissingToken)?;

        let token = bearer.token();

        #[cfg(debug_assertions)]
        if token == "test" {
            return Ok(Self {
                // uid: "test-user-id".to_string(),
                uid: "00000000-0000-0000-0000-000000000000".to_string(),
                email: Some("test@example.com".to_string()),
                is_admin: true,
            });
        }

        // Decode the header to get `kid`.
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        let kid = header.kid.ok_or(AuthError::InvalidToken)?;

        // Fetch (possibly cached) public keys.
        let mut keys = state
            .key_cache
            .get_keys(&state.http_client)
            .await
            .map_err(|_| AuthError::KeyFetchFailed)?;

        // If the kid isn't in cache, invalidate and retry once.
        if !keys.contains_key(&kid) {
            state.key_cache.invalidate().await;
            keys = state
                .key_cache
                .get_keys(&state.http_client)
                .await
                .map_err(|_| AuthError::KeyFetchFailed)?;
        }

        let pem = keys.get(&kid).ok_or(AuthError::InvalidToken)?;
        let decoding_key =
            DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| AuthError::InvalidToken)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&state.firebase_project_id]);

        let token_data = decode::<FirebaseClaims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        let claims = token_data.claims;

        // Check if this Firebase UID has admin privileges in our DB.
        let is_admin = sqlx::query_scalar!(
            "SELECT admin FROM users WHERE uuid = $1",
            claims.sub
        )
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .unwrap_or(false);

        Ok(AuthenticatedUser {
            uid: claims.sub,
            email: claims.email,
            is_admin,
        })
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Failed to fetch Firebase public keys")]
    KeyFetchFailed,
    #[error("Insufficient permissions")]
    Forbidden,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            AuthError::MissingToken | AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::KeyFetchFailed => StatusCode::SERVICE_UNAVAILABLE,
            AuthError::Forbidden => StatusCode::FORBIDDEN,
        };
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}
