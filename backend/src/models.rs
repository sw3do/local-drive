use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FileInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub disk_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub usage_percentage: u8,
    pub is_accessible: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub usage_percentage: u8,
    pub disk_count: usize,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageResult {
    pub file_id: Uuid,
    pub filename: String,
    pub file_path: String,
    pub disk_path: String,
    pub file_size: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChunkedUpload {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub total_size: i64,
    pub chunk_size: i64,
    pub total_chunks: i32,
    pub uploaded_chunks: i32,
    pub temp_path: String,
    pub disk_path: String,
    pub is_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateChunkedUploadRequest {
    pub filename: String,
    pub total_size: i64,
    pub chunk_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateChunkedUploadResponse {
    pub upload_id: Uuid,
    pub chunk_size: i64,
    pub total_chunks: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadChunkResponse {
    pub chunk_number: i32,
    pub uploaded: bool,
    pub upload_completed: bool,
    pub file_info: Option<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub chunk_number: i32,
    pub is_uploaded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TempFilesInfo {
    pub total_files: usize,
    pub total_size: u64,
    pub oldest_file_age_hours: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupResult {
    pub cleaned_files: usize,
    pub freed_space: u64,
}