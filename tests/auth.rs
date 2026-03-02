/// Интеграционные тесты для POST /auth/register и POST /auth/login.
mod common;

use todo_api::app::create_router;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

// ==================== REGISTER TESTS ====================

#[tokio::test]
async fn register_returns_201_with_token() {
    let state = common::test_app_state().await;
    let pool = state.db.clone();

    common::cleanup_user(&pool, "test@example.com").await;

    let app = create_router().with_state(state);

    let request = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "email": "test@example.com",
                "password": "password123"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("token").is_some(), "Response should contain 'token' field");

    common::cleanup_user(&pool, "test@example.com").await;
}

#[tokio::test]
async fn register_duplicate_email_returns_409() {
    let state = common::test_app_state().await;
    let pool = state.db.clone();

    common::cleanup_user(&pool, "duplicate@example.com").await;

    let app = create_router().with_state(state.clone());

    let body = serde_json::json!({
        "email": "duplicate@example.com",
        "password": "password123"
    })
    .to_string();

    // Первая регистрация — должна пройти.
    let req1 = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body.clone()))
        .unwrap();

    let resp1 = app.oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Вторая регистрация с тем же email — 409 Conflict.
    // Нужен новый роутер, т.к. oneshot потребляет Router.
    let app2 = create_router().with_state(state);

    let req2 = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap();

    let resp2 = app2.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CONFLICT);

    common::cleanup_user(&pool, "duplicate@example.com").await;
}

// ==================== LOGIN TESTS ====================

/// Вспомогательная: регистрирует пользователя через API для тестов логина.
async fn register_test_user(state: &todo_api::state::AppState, email: &str, password: &str) {
    let app = create_router().with_state(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/auth/register")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "Failed to register test user");
}

#[tokio::test]
async fn login_with_valid_credentials_returns_200() {
    let state = common::test_app_state().await;
    let pool = state.db.clone();
    let email = "login_valid@example.com";

    common::cleanup_user(&pool, email).await;
    register_test_user(&state, email, "password123").await;

    let app = create_router().with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "email": email,
                "password": "password123"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body.get("token").is_some(), "Response should contain 'token' field");

    common::cleanup_user(&pool, email).await;
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let state = common::test_app_state().await;
    let pool = state.db.clone();
    let email = "login_wrong_pw@example.com";

    common::cleanup_user(&pool, email).await;
    register_test_user(&state, email, "correct_password").await;

    let app = create_router().with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "email": email,
                "password": "wrong_password"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_user(&pool, email).await;
}

#[tokio::test]
async fn login_with_nonexistent_email_returns_401() {
    let state = common::test_app_state().await;

    let app = create_router().with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({
                "email": "nonexistent@example.com",
                "password": "whatever"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ==================== ME TESTS ====================

#[tokio::test]
async fn me_with_valid_token_returns_200_and_user_info() {
    let state = common::test_app_state().await;
    let pool = state.db.clone();
    let email = "me_valid@example.com";

    common::cleanup_user(&pool, email).await;
    let token = common::get_auth_token(&state, email).await;

    let app = create_router().with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header("Authorization", format!("Bearer {}", token))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body.get("id").is_some(), "Response should contain 'id' field");
    assert_eq!(body["email"].as_str().unwrap(), email);
    assert!(body.get("created_at").is_some(), "Response should contain 'created_at' field");

    common::cleanup_user(&pool, email).await;
}

#[tokio::test]
async fn me_without_token_returns_401() {
    let state = common::test_app_state().await;

    let app = create_router().with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_with_invalid_token_returns_401() {
    let state = common::test_app_state().await;

    let app = create_router().with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header("Authorization", "Bearer invalid.token.here")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
