mod test;

use axum::{debug_handler, extract::FromRequestParts};
use uuid::Uuid;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::{HeaderValue, Method};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use time::Duration;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

#[derive(Clone)]
struct Appstate {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env")
        .replacen("postgress://", "postgres://", 1);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    let session_store = PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("Session table migration failed");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(2)));

    let state = Appstate { db: pool };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([axum::http::header::CONTENT_TYPE])
        .allow_credentials(true);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/greet/{name}", get(greet_handle))
        .route("/db-check", get(db_check))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/{id}",
            axum::routing::patch(update_todo).delete(delete_todo),
        )
        .layer(session_layer)
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    println!("heALTH handle called");
    "ok"
}

async fn greet_handle(Path(name): Path<String>) -> impl IntoResponse {
    println!("grret handle called");
    format!("Hello {}", name)
}

async fn db_check(State(state): State<Appstate>) -> String {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
    {
        Ok(row) => format!("db says:{}", row),
        Err(e) => format!("query failed : {:?}", e),
    }
}

async fn me(AuthUser(user_id): AuthUser) -> String {
    format!("logged in as {}", user_id)
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

async fn register(
    State(state): State<Appstate>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let salt = SaltString::generate(&mut OsRng); //random salt per user
    let argon2 = Argon2::default();

    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to hash password").into_response();
        }
    };

    let result = sqlx::query("INSERT INTO users (username,password_hash) VALUES ($1,$2)")
        .bind(payload.username)
        .bind(password_hash)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => (StatusCode::CREATED, "user  created").into_response(),
        Err(e) => {
            eprintln!("Registor error: {:?}", e);
            (StatusCode::CONFLICT, "username may already be taken").into_response()
        }
    }
}

#[debug_handler]
async fn login(
    State(state): State<Appstate>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(user) = user else {
        return (StatusCode::UNAUTHORIZED, "invalid username or pasword").into_response();
    };

    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "corrupt password hash").into_response();
        }
    };

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response();
    }

    session
        .insert("user_id", user.id.to_string())
        .await
        .unwrap();

    (StatusCode::OK, "Logged in").into_response()
}

struct AuthUser(Uuid);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session error"))?;

        let user_id: Uuid = session
            .get("user_id")
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "session read error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "not logged in"))?;

        Ok(AuthUser(user_id))
    }
}

#[derive(Serialize)]
struct Todo {
    id: Uuid,
    title: String,
    completed: bool,
    #[serde(with = "time::serde::rfc3339")]
    created_at: time::OffsetDateTime,
}

#[debug_handler]
async fn list_todos(
    AuthUser(user_id): AuthUser,
    State(state): State<Appstate>,
) -> impl IntoResponse {
    let todos = sqlx::query_as!(Todo,"SELECT id,title,completed, created_at FROM  todos WHERE user_id = $1 ORDER BY created_at DESC",user_id).fetch_all(&state.db).await ;

    match todos {
        Ok(todos) => Json(todos).into_response(),
        Err(e) => {
            eprintln!("list todos error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch todos").into_response()
        }
    }
}

#[derive(Deserialize)]
struct CreateTodoRequest {
    title: String,
}
async fn create_todo(
    AuthUser(user_id): AuthUser,
    State(state): State<Appstate>,
    Json(payload): Json<CreateTodoRequest>,
) -> impl IntoResponse {
    let todo = sqlx::query_as!(
        Todo,
        "INSERT INTO todos (user_id,title) VALUES ($1,$2) RETURNING id,title,completed,created_at",
        user_id,
        payload.title
    )
    .fetch_one(&state.db)
    .await;

    match todo {
        Ok(todo) => (StatusCode::CREATED, Json(todo)).into_response(),
        Err(e) => {
            eprintln!("Created_todo error : {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to create todo").into_response()
        }
    }
}

#[derive(Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    completed: Option<bool>,
}
async fn update_todo(
    AuthUser(user_id): AuthUser,
    State(state): State<Appstate>,
    Path(todo_id): Path<Uuid>,
    Json(payload): Json<UpdateTodoRequest>,
) -> impl IntoResponse {
    let todo = sqlx::query_as!(
        Todo,
        r#"
    UPDATE todos 
    SET title= COALESCE($1,title),completed = COALESCE($2,completed)
    WHERE id = $3 AND user_id = $4 
    RETURNING id,title,completed,created_at"#,
        payload.title,
        payload.completed,
        todo_id,
        user_id,
    )
    .fetch_optional(&state.db)
    .await;

    match todo {
        Ok(Some(todo)) => Json(todo).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "todo not found").into_response(),
        Err(e) => {
            eprintln!("Update_todo error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update todo").into_response()
        }
    }
}

async fn delete_todo(
    AuthUser(user_id): AuthUser,
    State(state): State<Appstate>,
    Path(todo_id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "DELETE FROM todos WHERE id = $1 AND user_id = $2",
        todo_id,
        user_id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 1 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, "todo not found").into_response(),
        Err(e) => {
            eprintln!("Delete_todo error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete todo").into_response()
        }
    }
}
