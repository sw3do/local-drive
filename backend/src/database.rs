use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, FileInfo, ChunkedUpload};

pub async fn create_connection_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

pub async fn initialize_database(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(255) UNIQUE NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            is_admin BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            filename VARCHAR(255) NOT NULL,
            original_filename VARCHAR(255) NOT NULL,
            file_path VARCHAR(500) NOT NULL,
            disk_path VARCHAR(500) NOT NULL,
            file_size BIGINT NOT NULL,
            mime_type VARCHAR(255),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS shared_links (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            token VARCHAR(255) UNIQUE NOT NULL,
            expires_at TIMESTAMP WITH TIME ZONE,
            is_read_only BOOLEAN DEFAULT TRUE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chunked_uploads (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            filename VARCHAR(255) NOT NULL,
            total_size BIGINT NOT NULL,
            chunk_size BIGINT NOT NULL,
            total_chunks INTEGER NOT NULL,
            uploaded_chunks INTEGER DEFAULT 0,
            temp_path VARCHAR(500) NOT NULL,
            disk_path VARCHAR(500) NOT NULL,
            is_completed BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
    is_admin: bool,
) -> anyhow::Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, is_admin)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, password_hash, is_admin, created_at, updated_at
        "#,
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(is_admin)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> anyhow::Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin, created_at, updated_at FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> anyhow::Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin, created_at, updated_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, user_id: &Uuid) -> anyhow::Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_all_users(pool: &PgPool) -> anyhow::Result<Vec<User>> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_admin, created_at, updated_at FROM users ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}



pub async fn create_file_record(
    pool: &PgPool,
    user_id: &Uuid,
    filename: &str,
    original_filename: &str,
    file_path: &str,
    disk_path: &str,
    file_size: i64,
    mime_type: Option<&str>,
) -> anyhow::Result<FileInfo> {
    let file = sqlx::query_as::<_, FileInfo>(
        r#"
        INSERT INTO files (user_id, filename, original_filename, file_path, disk_path, file_size, mime_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, filename, original_filename, file_path, disk_path, file_size, mime_type, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(filename)
    .bind(original_filename)
    .bind(file_path)
    .bind(disk_path)
    .bind(file_size)
    .bind(mime_type)
    .fetch_one(pool)
    .await?;

    Ok(file)
}



pub async fn get_file_by_id(pool: &PgPool, file_id: &Uuid) -> anyhow::Result<Option<FileInfo>> {
    let file = sqlx::query_as::<_, FileInfo>(
        "SELECT id, user_id, filename, original_filename, file_path, disk_path, file_size, mime_type, created_at, updated_at FROM files WHERE id = $1",
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await?;

    Ok(file)
}

pub async fn delete_file_record(pool: &PgPool, file_id: &Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(file_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_all_files(pool: &PgPool) -> anyhow::Result<Vec<FileInfo>> {
    let files = sqlx::query_as::<_, FileInfo>(
        "SELECT id, user_id, filename, original_filename, file_path, disk_path, file_size, mime_type, created_at, updated_at FROM files ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(files)
}



pub async fn create_chunked_upload(
    pool: &PgPool,
    user_id: &Uuid,
    filename: &str,
    total_size: i64,
    chunk_size: i64,
    total_chunks: i32,
    temp_path: &str,
    disk_path: &str,
) -> anyhow::Result<ChunkedUpload> {
    let upload = sqlx::query_as::<_, ChunkedUpload>(
        r#"
        INSERT INTO chunked_uploads (user_id, filename, total_size, chunk_size, total_chunks, temp_path, disk_path)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, filename, total_size, chunk_size, total_chunks, uploaded_chunks, temp_path, disk_path, is_completed, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(filename)
    .bind(total_size)
    .bind(chunk_size)
    .bind(total_chunks)
    .bind(temp_path)
    .bind(disk_path)
    .fetch_one(pool)
    .await?;

    Ok(upload)
}

pub async fn get_chunked_upload(pool: &PgPool, upload_id: &Uuid) -> anyhow::Result<Option<ChunkedUpload>> {
    let upload = sqlx::query_as::<_, ChunkedUpload>(
        "SELECT id, user_id, filename, total_size, chunk_size, total_chunks, uploaded_chunks, temp_path, disk_path, is_completed, created_at, updated_at FROM chunked_uploads WHERE id = $1"
    )
    .bind(upload_id)
    .fetch_optional(pool)
    .await?;

    Ok(upload)
}

pub async fn update_chunked_upload_progress(
    pool: &PgPool,
    upload_id: &Uuid,
    uploaded_chunks: i32,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE chunked_uploads SET uploaded_chunks = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(uploaded_chunks)
    .bind(upload_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn complete_chunked_upload(
    pool: &PgPool,
    upload_id: &Uuid,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE chunked_uploads SET is_completed = TRUE, updated_at = NOW() WHERE id = $1"
    )
    .bind(upload_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_chunked_upload(
    pool: &PgPool,
    upload_id: &Uuid,
) -> anyhow::Result<()> {
    sqlx::query(
        "DELETE FROM chunked_uploads WHERE id = $1"
    )
    .bind(upload_id)
    .execute(pool)
    .await?;

    Ok(())
}