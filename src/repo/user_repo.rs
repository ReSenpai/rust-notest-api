use sqlx::PgPool;

use crate::models::user::User;

/// Создаёт нового пользователя в БД.
pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str) -> sqlx::Result<User> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Ищет пользователя по email.
pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

/// Ищет пользователя по ID.
pub async fn find_by_id(pool: &PgPool, id: uuid::Uuid) -> sqlx::Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}
