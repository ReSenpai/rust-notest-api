# 🗂 todo-api

Backend-сервис для управления TODO-списками и задачами на Rust.

> **Учебный проект** — разработка ведётся пошагово по TDD (Test-Driven Development).

---

## 🎯 Цель

- Регистрация и авторизация пользователей (JWT + Argon2)
- CRUD TODO-листов (у каждого пользователя свои списки)
- CRUD задач внутри листов
- Статусы задач: `todo` → `in_progress` → `done`
- PostgreSQL как хранилище
- Чистая слоёная архитектура

---

## ✅ Прогресс

### Инфраструктура
- [x] Docker Compose (PostgreSQL + Adminer + API)
- [x] Dockerfile (multi-stage build)
- [x] Автоматические миграции при старте (`sqlx::migrate!`)
- [x] Миграция: таблица `users`
- [x] Миграция: таблица `todo_lists`
- [x] Миграция: таблица `tasks`

### CI/CD
- [x] GitHub Actions: CI — тесты на push в `dev` и `main`
- [x] GitHub Actions: CD — сборка, GHCR, деплой на сервер (push в `main`)
- [x] `docker-compose.prod.yml` — продакшн конфиг (образ из GHCR)

### Auth
- [x] `POST /auth/register` — регистрация
- [x] `POST /auth/login` — вход (JWT-токен)
- [x] `GET /auth/me` — проверка авторизации по токену
- [x] Argon2 хеширование паролей
- [x] Генерация JWT (HS256, 24ч)
- [x] Интеграционные тесты auth (8 тестов)
- [x] JWT middleware (защита маршрутов) + 3 теста

### Health
- [x] `GET /health` — проверка жизни сервиса
- [x] Интеграционный тест health

### TODO Lists
- [x] Модель `TodoList`
- [x] DTO для списков
- [x] `list_repo` — CRUD в БД
- [x] `list_service` — бизнес-логика
- [x] Маршруты: `POST / GET / PUT / DELETE /lists`
- [x] Интеграционные тесты lists (7 тестов)

### Tasks
- [x] Модель `Task` (статусы: `todo`, `in_progress`, `done`)
- [x] DTO для задач
- [x] `task_repo` — CRUD в БД
- [x] `task_service` — бизнес-логика (с проверкой владения списком)
- [x] Маршруты: `POST / GET / PUT / DELETE /lists/:id/tasks`
- [x] Интеграционные тесты tasks (7 тестов)

### Документация
- [x] Swagger UI (utoipa) — интерактивная документация API
- [x] OpenAPI 3.1 спецификация (`/api-docs/openapi.json`)
- [x] Bearer-токен авторизация в Swagger UI

---

## 🧱 Архитектура проекта

```
todo-api/
├── Cargo.toml                 # зависимости проекта (Rust 2024 edition)
├── Cargo.lock                 # зафиксированные версии зависимостей
├── Dockerfile                 # multi-stage сборка (builder + runtime)
├── docker-compose.yml         # PostgreSQL + Adminer + API (dev)
├── docker-compose.prod.yml    # PostgreSQL + API из GHCR (production)
├── .dockerignore              # исключения для Docker-контекста
├── .env                       # переменные окружения (DATABASE_URL, JWT_SECRET)
├── requests.http              # готовые HTTP-запросы для тестирования
├── .github/
│   └── workflows/
│       ├── ci.yml             # CI: тесты на push в dev/main
│       └── deploy.yml         # CD: сборка → GHCR → SSH deploy
├── migrations/
│   ├── *_create_users_table.sql
│   ├── *_create_todo_lists_table.up.sql
│   └── *_create_tasks_table.up.sql
├── src/
│   ├── main.rs                # точка входа: PgPool, миграции, запуск сервера
│   ├── app.rs                 # create_router() — сборка всех маршрутов
│   ├── lib.rs                 # re-export модулей (pub mod ...)
│   ├── state.rs               # AppState { db, jwt_secret }
│   ├── errors.rs              # AppError — единая обработка ошибок (401/404/409/422/500)
│   ├── middleware/
│   │   └── auth.rs            # AuthUser extractor — проверка JWT из заголовка
│   ├── routes/
│   │   ├── auth.rs            # POST /auth/register, /auth/login, GET /auth/me
│   │   ├── health.rs          # GET /health
│   │   ├── lists.rs           # POST/GET/PUT/DELETE /lists
│   │   └── tasks.rs           # POST/GET/PUT/DELETE /lists/:id/tasks
│   ├── handlers/
│   │   ├── auth.rs            # обработка HTTP-запросов auth
│   │   ├── health.rs          # обработка health check
│   │   ├── lists.rs           # обработка CRUD списков (с AuthUser)
│   │   └── tasks.rs           # обработка CRUD задач (с AuthUser)
│   ├── services/
│   │   ├── auth.rs            # Argon2, JWT create/validate
│   │   ├── lists.rs           # бизнес-логика списков
│   │   └── tasks.rs           # бизнес-логика задач + verify_list_ownership
│   ├── repo/
│   │   ├── user_repo.rs       # SQL: create_user, find_by_email
│   │   ├── list_repo.rs       # SQL: CRUD todo_lists
│   │   └── task_repo.rs       # SQL: CRUD tasks
│   ├── models/
│   │   ├── user.rs            # User { id, email, password_hash, created_at }
│   │   ├── todo_list.rs       # TodoList { id, user_id, title, timestamps }
│   │   └── task.rs            # Task { id, list_id, title, status, timestamps }
│   └── dto/
│       ├── auth.rs            # RegisterRequest, LoginRequest, AuthResponse, MeResponse
│       ├── lists.rs           # CreateListRequest, UpdateListRequest, ListResponse
│       └── tasks.rs           # CreateTaskRequest, UpdateTaskRequest, TaskResponse
├── tests/
│   ├── common/mod.rs          # test_app_state(), cleanup_user()
│   ├── health.rs              # 1 тест
│   ├── auth.rs                # 8 тестов
│   ├── middleware_auth.rs     # 3 теста
│   ├── lists.rs               # 7 тестов
│   └── tasks.rs               # 7 тестов
└── README.md
```

---

## 🧠 Разделение ответственности

| Слой         | Отвечает за                              |
|--------------|------------------------------------------|
| `routes`     | URL + HTTP метод                         |
| `handlers`   | HTTP → вызов сервиса                     |
| `middleware`  | JWT-авторизация (AuthUser extractor)    |
| `services`   | Бизнес-логика                            |
| `repo`       | SQL-запросы (PostgreSQL)                 |
| `models`     | Доменная модель (ORM-маппинг)            |
| `dto`        | JSON вход/выход (request/response)       |
| `state`      | Shared-зависимости (DB, JWT)             |
| `errors`     | Единый error flow (AppError → HTTP)      |

---

## 🧰 Технологии

| Категория       | Стек                                      |
|-----------------|-------------------------------------------|
| Web framework   | axum                                      |
| Async runtime   | tokio                                     |
| Сериализация    | serde + serde_json                        |
| База данных     | PostgreSQL + sqlx                         |
| Авторизация     | JWT (jsonwebtoken) + Argon2               |
| Конфигурация    | .env (dotenvy)                            |
| Логирование     | tracing + tracing-subscriber              |
| Ошибки          | AppError (thiserror)                      |
| Тестирование    | Integration tests (TDD), 26 тестов         |
| Инфраструктура  | Docker (multi-stage), docker-compose      |
| CI/CD           | GitHub Actions, GHCR, SSH deploy          |
| Документация     | utoipa + Swagger UI (OpenAPI 3.1)         |

---

## 🧪 Подход: TDD

Разработка каждой фичи идёт по циклу:

1. **Red** — пишем тест, который падает
2. **Green** — пишем минимальный код, чтобы тест прошёл
3. **Refactor** — улучшаем код, тесты остаются зелёными

```bash
cargo test                       # все 26 тестов
cargo test --test auth           # 8 тестов auth
cargo test --test health         # 1 тест health
cargo test --test middleware_auth # 3 теста middleware
cargo test --test lists          # 7 тестов lists
cargo test --test tasks          # 7 тестов tasks
```

---

## 🚀 Запуск

### Всё в Docker (production-ready)

```bash
docker-compose up --build    # PostgreSQL + Adminer + API — всё из коробки
```

| Сервис     | URL                           |
|----------|-------------------------------|
| API      | http://localhost:3000          |
| Swagger  | http://localhost:3000/swagger-ui/ |
| OpenAPI  | http://localhost:3000/api-docs/openapi.json |
| Adminer  | http://localhost:8080          |
| Postgres | localhost:5432                 |

### Локальная разработка

```bash
docker-compose up postgres adminer -d   # только БД + Adminer
sqlx migrate run                        # применить миграции
cargo run                               # запустить API
```

---

## 🔄 CI/CD

### Пайплайн

```
push в dev  ─────────────────────►  CI: тесты
push в main ──► CI: тесты ──► Build & Push (GHCR) ──► Deploy (SSH)
```

### CI (`.github/workflows/ci.yml`)

- Запускается на push в `dev`, `main` и на PR в `main`
- Поднимает PostgreSQL service container
- Устанавливает Rust, кэширует cargo, прогоняет миграции
- Запускает `cargo test`

### CD (`.github/workflows/deploy.yml`)

- Запускается только на push в `main`
- **Job 1** — тесты (обязательно перед деплоем)
- **Job 2** — сборка Docker-образа → push в `ghcr.io`
- **Job 3** — SSH на сервер → `docker compose pull` → `docker compose up -d`

### GitHub Secrets

Настрой в: **Settings → Secrets and variables → Actions**

| Secret              | Описание                                                   |
|---------------------|------------------------------------------------------------|
| `SERVER_HOST`       | IP-адрес Ubuntu-сервера                                    |
| `SERVER_USER`       | SSH-пользователь (например `deploy`)                        |
| `SERVER_SSH_KEY`    | Приватный SSH-ключ (весь файл `id_ed25519`)                |
| `POSTGRES_PASSWORD` | Пароль PostgreSQL на продакшне                              |
| `JWT_SECRET`        | Секрет для подписи JWT на продакшне                         |
| `GHCR_PAT`         | Personal Access Token с `read:packages` (для docker login) |

### Подготовка сервера

```bash
# 1. Установи Docker + Docker Compose v2
# 2. Создай директорию проекта
mkdir -p ~/todo-api

# 3. Скопируй docker-compose.prod.yml на сервер
scp docker-compose.prod.yml user@server:~/todo-api/docker-compose.yml

# Всё остальное (pull, .env, запуск) делает CD-пайплайн автоматически.
```