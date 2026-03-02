use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::repo::user_repo;

/// Claims — содержимое JWT-токена.
///
/// JWT состоит из трёх частей: header.payload.signature
/// `Claims` — это payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — идентификатор пользователя (UUID в виде строки).
    pub sub: String,
    /// Expiration — время истечения токена (Unix timestamp).
    pub exp: usize,
}

/// Регистрация нового пользователя.
///
/// Алгоритм:
/// 1. Проверяем, не занят ли email
/// 2. Хэшируем пароль через argon2
/// 3. Сохраняем пользователя в БД
/// 4. Генерируем JWT-токен
///
/// Возвращает JWT-токен или ошибку.
pub async fn register(
    pool: &PgPool,
    jwt_secret: &str,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let existing = user_repo::find_by_email(pool, email).await?;
    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    //    Хэшируем пароль.
    //    Argon2 — один из лучших алгоритмов хэширования паролей.
    //    Salt (соль) генерируется случайно для каждого пароля —
    //    это защищает от rainbow table атак.
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::Validation("Failed to hash password".to_string()))?
        .to_string();

    let user = user_repo::create_user(pool, email, &password_hash).await?;

    //    Генерируем JWT-токен.
    let token = create_jwt(&user.id.to_string(), jwt_secret)?;

    Ok(token)
}

/// Логин существующего пользователя.
///
/// Алгоритм:
/// 1. Ищем пользователя по email
/// 2. Если не нашли — Unauthorized (не говорим "email не найден"!)
/// 3. Проверяем пароль через argon2 verify
/// 4. Если пароль неверный — Unauthorized
/// 5. Генерируем JWT-токен
pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let user = user_repo::find_by_email(pool, email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Unauthorized)?;

    // Верифицируем пароль — argon2 сравнивает введённый пароль с хэшем из БД.
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let token = create_jwt(&user.id.to_string(), jwt_secret)?;

    Ok(token)
}

/// Создаёт JWT-токен для пользователя.
///
/// Токен действителен 24 часа. Содержит `sub` (user_id) и `exp` (expiration).
/// Подписывается секретным ключом (HMAC-SHA256).
fn create_jwt(user_id: &str, secret: &str) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    // `encode` подписывает claims секретным ключом и возвращает строку
    // вида "eyJhbGciOi..."
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Validation("Failed to create token".to_string()))?;

    Ok(token)
}

/// Валидирует JWT-токен и возвращает claims.
pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    Ok(token_data.claims)
}

/// Возвращает информацию о текущем пользователе по его ID из токена.
pub async fn me(pool: &PgPool, user_id: &str) -> Result<crate::dto::auth::MeResponse, AppError> {
    let uuid = uuid::Uuid::parse_str(user_id)
        .map_err(|_| AppError::Unauthorized)?;

    let user = user_repo::find_by_id(pool, uuid)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(crate::dto::auth::MeResponse {
        id: user.id.to_string(),
        email: user.email,
        created_at: user.created_at.map(|dt| dt.to_rfc3339()),
    })
}
