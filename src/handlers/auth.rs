use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::dto::auth::{AuthResponse, LoginRequest, MeResponse, RegisterRequest};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::services;
use crate::state::AppState;

/// POST /auth/register — регистрация нового пользователя.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Пользователь создан", body = AuthResponse),
        (status = 409, description = "Email уже занят", body = crate::dto::ErrorResponse),
        (status = 422, description = "Невалидные данные", body = crate::dto::ErrorResponse)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let token = services::auth::register(&state.db, &state.jwt_secret, &body.email, &body.password)
        .await?;

    // 201 Created — стандартный код для успешного создания ресурса.
    Ok((StatusCode::CREATED, Json(AuthResponse { token })))
}

/// POST /auth/login — вход существующего пользователя.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Успешный вход", body = AuthResponse),
        (status = 401, description = "Неверные учётные данные", body = crate::dto::ErrorResponse)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let token = services::auth::login(&state.db, &state.jwt_secret, &body.email, &body.password)
        .await?;

    // 200 OK — возвращается автоматически для Json<T> без явного StatusCode.
    Ok(Json(AuthResponse { token }))
}

/// GET /auth/me — информация о текущем пользователе.
///
/// Требует валидный JWT-токен в заголовке `Authorization: Bearer <token>`.
/// Возвращает id, email и дату регистрации пользователя.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Информация о пользователе", body = MeResponse),
        (status = 401, description = "Невалидный или отсутствующий токен", body = crate::dto::ErrorResponse)
    )
)]
pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<MeResponse>, AppError> {
    let response = services::auth::me(&state.db, &auth_user.user_id).await?;
    Ok(Json(response))
}
