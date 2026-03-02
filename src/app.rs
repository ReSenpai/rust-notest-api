use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::routes;
use crate::state::AppState;

/// OpenAPI-спецификация — собирает описание всех эндпоинтов и схем.
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health
        handlers::health::health_check,
        // Auth
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::me,
        // Lists
        handlers::lists::create,
        handlers::lists::get_all,
        handlers::lists::get_one,
        handlers::lists::update,
        handlers::lists::delete,
        // Tasks
        handlers::tasks::create,
        handlers::tasks::get_all,
        handlers::tasks::get_one,
        handlers::tasks::update,
        handlers::tasks::delete,
    ),
    components(
        schemas(
            // Auth
            crate::dto::auth::RegisterRequest,
            crate::dto::auth::LoginRequest,
            crate::dto::auth::AuthResponse,
            crate::dto::auth::MeResponse,
            // Lists
            crate::dto::lists::CreateListRequest,
            crate::dto::lists::UpdateListRequest,
            crate::dto::lists::ListResponse,
            // Tasks
            crate::dto::tasks::CreateTaskRequest,
            crate::dto::tasks::UpdateTaskRequest,
            crate::dto::tasks::TaskResponse,
            // Errors
            crate::dto::ErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Проверка жизни сервиса"),
        (name = "Auth", description = "Регистрация и авторизация (JWT)"),
        (name = "Lists", description = "CRUD TODO-листов"),
        (name = "Tasks", description = "CRUD задач внутри списков")
    )
)]
pub struct ApiDoc;

/// Добавляет Bearer-токен авторизацию в Swagger UI.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(utoipa::openapi::security::Http::new(
                utoipa::openapi::security::HttpAuthScheme::Bearer,
            )),
        );
    }
}

/// Создаёт и возвращает основной Router приложения.
pub fn create_router() -> Router<AppState> {
    Router::new()
        .merge(routes::health::router())
        .merge(routes::auth::router())
        .merge(routes::lists::router())
        .merge(routes::tasks::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
