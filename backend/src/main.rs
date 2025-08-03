use axum::{
    extract::{Path, State, Extension},
    http::{StatusCode, Method, HeaderValue, header},
    middleware,
    response::{Json, Response},
    routing::{delete, get, post},
    Router,
    body::Body,
};
use axum::body::Bytes;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::info;
use uuid::Uuid;
use clap::{Parser, Subcommand};
use tokio_cron_scheduler::{JobScheduler, Job};

mod auth;
mod config;
mod database;
mod file_storage;
mod models;

use config::Config;
use models::*;

#[derive(Parser)]
#[command(name = "local-drive-backend")]
#[command(about = "A self-hosted file storage backend")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    CreateAdmin {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
    },
    Serve,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub file_storage: Arc<file_storage::FileStorage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = Config::from_env()?;
    let db = database::create_connection_pool(&config.database_url).await?;
    database::initialize_database(&db).await?;

    match cli.command {
        Some(Commands::CreateAdmin { username, email, password }) => {
            create_admin_user(&db, &username, &email, &password).await?;
            return Ok(());
        }
        Some(Commands::Serve) | None => {
        }
    }

    let file_storage = Arc::new(file_storage::FileStorage::new(&config)?);
    let state = AppState { db, config: config.clone(), file_storage };

    let scheduler = JobScheduler::new().await?;
    let file_storage_clone = state.file_storage.clone();
    
    let cleanup_job = Job::new_async("0 0 */6 * * *", move |_uuid, _l| {
        let file_storage = file_storage_clone.clone();
        Box::pin(async move {
            if let Ok(result) = file_storage.cleanup_orphaned_temp_files() {
                info!("Automatic temp cleanup: {} files removed, {} bytes freed", result.cleaned_files, result.freed_space);
            }
        })
    })?;
    
    scheduler.add(cleanup_job).await?;
    scheduler.start().await?;
    
    info!("Automatic temp file cleanup scheduled (every 6 hours)");

    let protected_routes = Router::new()
        .route("/files", get(list_files))
        .route("/files/:id/download", get(download_file))
        .route("/files/:id", delete(move_to_trash))
        .route("/trash", get(list_trash_files))
        .route("/trash/:id/restore", post(restore_file))
        .route("/trash/:id", delete(delete_file_permanently))
        .route("/upload/initiate", post(initiate_chunked_upload))
        .route("/upload/:upload_id/chunk/:chunk_number", post(upload_chunk))
        .route("/upload/:upload_id/complete", post(complete_chunked_upload))
        .route("/upload/:upload_id/status", get(get_upload_status))
        .route("/upload/:upload_id/cancel", delete(cancel_chunked_upload))
        .route("/user/storage", get(get_user_storage_info))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware));

    let admin_routes = Router::new()
        .route("/admin/users", get(list_users))
        .route("/admin/storage", get(get_storage_info))
        .route("/admin/storage/report", get(get_disk_usage_report))
        .route("/admin/temp/info", get(get_temp_files_info))
        .route("/admin/temp/cleanup", post(cleanup_temp_files))
        .route("/admin/temp/cleanup/:hours", post(cleanup_temp_files_with_age))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::admin_middleware));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login))
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                .expose_headers([header::CONTENT_DISPOSITION, header::CONTENT_LENGTH])
                .allow_credentials(true)
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    info!("Server running on port {}", config.port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_admin_user(
    db: &PgPool,
    username: &str,
    email: &str,
    password: &str,
) -> anyhow::Result<()> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::{rand_core::OsRng, SaltString};

    let existing_user = database::get_user_by_username(db, username).await?;
    if existing_user.is_some() {
        println!("Admin user '{}' already exists!", username);
        return Ok(());
    }

    let existing_email = database::get_user_by_email(db, email).await?;
    if existing_email.is_some() {
        println!("User with email '{}' already exists!", email);
        return Ok(());
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    let user = database::create_user(db, username, email, &password_hash, true).await?;
    println!("Admin user created successfully!");
    println!("Username: {}", user.username);
    println!("Email: {}", user.email);
    println!("User ID: {}", user.id);

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now()
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = database::get_user_by_username(&state.db, &request.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth::verify_password(&request.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = auth::create_jwt_token(&user.id, &user.username, user.is_admin, &state.config.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse { token, user }))
}



async fn list_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<FileInfo>>, StatusCode> {
    let files = database::get_all_files(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(files))
}

async fn download_file(
    Path(file_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user): Extension<models::User>,
) -> Result<Response<Body>, StatusCode> {
    let file = database::get_file_by_id(&state.db, &file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if file.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let file_data = state.file_storage
        .get_file_data(&file.file_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let content_type = file.mime_type
        .as_deref()
        .unwrap_or("application/octet-stream");

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.original_filename)
        )
        .header(header::CONTENT_LENGTH, file_data.len())
        .body(Body::from(file_data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

async fn move_to_trash(
    Path(file_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    database::soft_delete_file(&state.db, &file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_trash_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<FileInfo>>, StatusCode> {
    let files = database::get_deleted_files(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(files))
}

async fn restore_file(
    Path(file_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    database::restore_file(&state.db, &file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_file_permanently(
    Path(file_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let file = database::get_file_by_id(&state.db, &file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if !file.is_deleted {
        return Err(StatusCode::BAD_REQUEST);
    }

    state.file_storage.delete_file(&file.file_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    database::delete_file_record(&state.db, &file_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let users = database::get_all_users(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

async fn get_storage_info(
    State(state): State<AppState>,
) -> Result<Json<StorageInfo>, StatusCode> {
    let storage_info = state.file_storage.get_storage_info()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(storage_info))
}

async fn get_user_storage_info(
    State(state): State<AppState>,
) -> Result<Json<StorageInfo>, StatusCode> {
    let storage_info = state.file_storage.get_storage_info()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(storage_info))
}

async fn initiate_chunked_upload(
    State(state): State<AppState>,
    Extension(user): Extension<models::User>,
    Json(request): Json<models::InitiateChunkedUploadRequest>,
) -> Result<Json<models::InitiateChunkedUploadResponse>, StatusCode> {
    let user_id = user.id;
    
    let total_chunks = (request.total_size as f64 / request.chunk_size as f64).ceil() as i32;
    let upload_id = Uuid::new_v4();
    
    let (temp_file_path, disk_path) = state.file_storage
        .create_temp_file(&user_id, &upload_id, request.total_size as u64)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let upload = database::create_chunked_upload(
        &state.db,
        &user_id,
        &request.filename,
        request.total_size,
        request.chunk_size,
        total_chunks,
        &temp_file_path.to_string_lossy(),
        &disk_path.to_string_lossy(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(models::InitiateChunkedUploadResponse {
        upload_id: upload.id,
        chunk_size: upload.chunk_size,
        total_chunks: upload.total_chunks,
    }))
}

async fn upload_chunk(
    Path((upload_id, chunk_number)): Path<(Uuid, i32)>,
    State(state): State<AppState>,
    Extension(user): Extension<models::User>,
    body: Bytes,
) -> Result<Json<models::UploadChunkResponse>, StatusCode> {
    let upload = database::get_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if upload.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    if upload.is_completed {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let temp_file_path = std::path::Path::new(&upload.temp_path);
    
    state.file_storage
        .write_chunk(temp_file_path, &body, chunk_number, upload.chunk_size)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let new_uploaded_chunks = upload.uploaded_chunks + 1;
    
    database::update_chunked_upload_progress(&state.db, &upload_id, new_uploaded_chunks)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let upload_completed = new_uploaded_chunks >= upload.total_chunks;
    
    Ok(Json(models::UploadChunkResponse {
        chunk_number,
        uploaded: true,
        upload_completed,
        file_info: None,
    }))
}

async fn complete_chunked_upload(
    Path(upload_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user): Extension<models::User>,
) -> Result<Json<models::FileInfo>, StatusCode> {
    let upload = database::get_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if upload.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    if upload.uploaded_chunks < upload.total_chunks {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let temp_file_path = std::path::Path::new(&upload.temp_path);
    let disk_path = std::path::Path::new(&upload.disk_path);
    
    let storage_result = state.file_storage
        .finalize_chunked_upload(temp_file_path, &upload.user_id, &upload.filename, disk_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let file_info = database::create_file_record(
        &state.db,
        &upload.user_id,
        &storage_result.filename,
        &upload.filename,
        &storage_result.file_path,
        &storage_result.disk_path,
        storage_result.file_size,
        None,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    database::complete_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    database::delete_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(file_info))
}

async fn get_upload_status(
    Path(upload_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user): Extension<models::User>,
) -> Result<Json<models::ChunkedUpload>, StatusCode> {
    let upload = database::get_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if upload.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(Json(upload))
}

async fn cancel_chunked_upload(
    Path(upload_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user): Extension<models::User>,
) -> Result<StatusCode, StatusCode> {
    let upload = database::get_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    if upload.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let temp_file_path = std::path::Path::new(&upload.temp_path);
    state.file_storage.cleanup_temp_file(temp_file_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    database::delete_chunked_upload(&state.db, &upload_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(StatusCode::NO_CONTENT)
}

async fn get_disk_usage_report(
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    let report = state.file_storage.get_disk_usage_report()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(report)
}

async fn get_temp_files_info(
    State(state): State<AppState>,
) -> Result<Json<models::TempFilesInfo>, StatusCode> {
    let temp_info = state.file_storage
        .get_temp_files_info()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(temp_info))
}

async fn cleanup_temp_files(
    State(state): State<AppState>,
) -> Result<Json<models::CleanupResult>, StatusCode> {
    let result = state.file_storage.cleanup_orphaned_temp_files()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}

async fn cleanup_temp_files_with_age(
    Path(hours): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<models::CleanupResult>, StatusCode> {
    let result = state.file_storage.cleanup_old_temp_files(hours)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}
