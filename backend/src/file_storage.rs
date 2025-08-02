use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read, Seek, SeekFrom};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::models::{DiskInfo, StorageInfo, StorageResult};
use crate::config::Config;

pub struct FileStorage {
    pub storage_paths: Vec<PathBuf>,
}

impl FileStorage {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let mut storage_paths = Vec::new();
        
        for path_str in &config.storage_paths {
            let path = PathBuf::from(path_str);
            let normalized_path = Self::normalize_path(&path)?;
            
            if !normalized_path.exists() {
                fs::create_dir_all(&normalized_path)?;
            }
            storage_paths.push(normalized_path);
        }
        
        Ok(FileStorage { storage_paths })
    }
    
    fn normalize_path(path: &Path) -> anyhow::Result<PathBuf> {
        if cfg!(target_os = "windows") {
            let path_str = path.to_string_lossy();
            let normalized = path_str.replace('/', "\\\\")
                .trim_end_matches('\\').to_string();
            Ok(PathBuf::from(normalized))
        } else {
            Ok(path.to_path_buf())
        }
    }
    
    pub fn get_disk_info(&self) -> anyhow::Result<Vec<DiskInfo>> {
        let mut disk_infos = Vec::new();
        
        for (index, path) in self.storage_paths.iter().enumerate() {
            let disk_info = self.get_single_disk_info(path, index)?;
            disk_infos.push(disk_info);
        }
        
        Ok(disk_infos)
    }
    
    fn get_single_disk_info(&self, path: &Path, _index: usize) -> anyhow::Result<DiskInfo> {
        let canonical_path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };
        
        let metadata = fs::metadata(&canonical_path).or_else(|_| fs::metadata(path))?;
        
        let (total_space, available_space) = if cfg!(target_os = "windows") {
            self.get_windows_disk_space(&canonical_path)?
        } else {
            self.get_unix_disk_space(&canonical_path)?
        };
        
        let used_space = total_space.saturating_sub(available_space);
        let usage_percentage = if total_space > 0 {
            ((used_space as f64 / total_space as f64) * 100.0).min(100.0) as u8
        } else {
            0
        };
        
        Ok(DiskInfo {
            path: canonical_path.to_string_lossy().to_string(),
            total_space,
            used_space,
            available_space,
            usage_percentage,
            is_accessible: canonical_path.exists() && metadata.is_dir(),
        })
    }
    
    #[cfg(target_os = "windows")]
    fn get_windows_disk_space(&self, path: &Path) -> anyhow::Result<(u64, u64)> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::fileapi::GetDiskFreeSpaceExW;
        use winapi::shared::minwindef::BOOL;
        
        let path_wide: Vec<u16> = OsStr::new(&path.to_string_lossy())
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        let mut free_bytes_available = 0u64;
        let mut total_number_of_bytes = 0u64;
        let mut total_number_of_free_bytes = 0u64;
        
        let result: BOOL = unsafe {
            GetDiskFreeSpaceExW(
                path_wide.as_ptr(),
                &mut free_bytes_available,
                &mut total_number_of_bytes,
                &mut total_number_of_free_bytes,
            )
        };
        
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to get disk space information for Windows"));
        }
        
        Ok((total_number_of_bytes, free_bytes_available))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn get_unix_disk_space(&self, path: &Path) -> anyhow::Result<(u64, u64)> {
        use std::ffi::CString;
        use std::mem;
        
        let path_cstring = CString::new(path.to_string_lossy().as_bytes())?;
        let mut statvfs: libc::statvfs = unsafe { mem::zeroed() };
        
        let result = unsafe { libc::statvfs(path_cstring.as_ptr(), &mut statvfs) };
        
        if result != 0 {
            return Err(anyhow::anyhow!("Failed to get disk space information"));
        }
        
        let total_space = (statvfs.f_blocks as u64) * (statvfs.f_frsize as u64);
        let available_space = (statvfs.f_bavail as u64) * (statvfs.f_frsize as u64);
        
        Ok((total_space, available_space))
    }
    
    #[cfg(target_os = "windows")]
    fn get_unix_disk_space(&self, _path: &Path) -> anyhow::Result<(u64, u64)> {
        Ok((0, 0))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn get_windows_disk_space(&self, _path: &Path) -> anyhow::Result<(u64, u64)> {
        Ok((0, 0))
    }
    
    pub fn get_storage_info(&self) -> anyhow::Result<StorageInfo> {
        let disk_infos = self.get_disk_info()?;
        
        let total_space = disk_infos.iter().map(|d| d.total_space).sum();
        let used_space = disk_infos.iter().map(|d| d.used_space).sum();
        let available_space = disk_infos.iter().map(|d| d.available_space).sum();
        
        let usage_percentage = if total_space > 0 {
            (used_space as f64 / total_space as f64 * 100.0) as u8
        } else {
            0
        };
        
        Ok(StorageInfo {
            total_space,
            used_space,
            available_space,
            usage_percentage,
            disk_count: disk_infos.len(),
            disks: disk_infos,
        })
    }
    
    pub fn get_disk_usage_report(&self) -> anyhow::Result<String> {
        let disk_infos = self.get_disk_info()?;
        let mut report = String::new();
        
        report.push_str("Disk Usage Report:\n");
        report.push_str(&format!("Total Disks: {}\n", disk_infos.len()));
        
        for (index, disk) in disk_infos.iter().enumerate() {
            let gb_total = disk.total_space as f64 / (1024.0 * 1024.0 * 1024.0);
            let gb_available = disk.available_space as f64 / (1024.0 * 1024.0 * 1024.0);
            let gb_used = disk.used_space as f64 / (1024.0 * 1024.0 * 1024.0);
            
            report.push_str(&format!(
                "Disk {}: {}\n  Total: {:.2} GB\n  Used: {:.2} GB\n  Available: {:.2} GB\n  Usage: {}%\n  Accessible: {}\n\n",
                index + 1,
                disk.path,
                gb_total,
                gb_used,
                gb_available,
                disk.usage_percentage,
                disk.is_accessible
            ));
        }
        
        Ok(report)
    }
    
    pub fn find_available_disk(&self, file_size: u64) -> anyhow::Result<Option<PathBuf>> {
        let mut best_disk: Option<(PathBuf, u64)> = None;
        let min_free_space_buffer = 1024 * 1024 * 100;
        
        for path in &self.storage_paths {
            let disk_info = self.get_single_disk_info(path, 0)?;
            
            if disk_info.is_accessible && 
               disk_info.available_space > file_size + min_free_space_buffer {
                
                match &best_disk {
                    None => {
                        best_disk = Some((path.clone(), disk_info.available_space));
                    }
                    Some((_, current_best_space)) => {
                        if disk_info.available_space > *current_best_space {
                            best_disk = Some((path.clone(), disk_info.available_space));
                        }
                    }
                }
            }
        }
        
        Ok(best_disk.map(|(path, _)| path))
    }
    
    pub fn store_file(
        &self,
        file_data: &[u8],
        user_id: &Uuid,
        original_filename: &str,
    ) -> anyhow::Result<StorageResult> {
        let file_size = file_data.len() as u64;
        
        let disk_path = match self.find_available_disk(file_size)? {
            Some(path) => path,
            None => {
                return Err(anyhow::anyhow!("No available disk space for file"));
            }
        };
        
        let file_id = Uuid::new_v4();
        let file_extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let filename = if file_extension.is_empty() {
            file_id.to_string()
        } else {
            format!("{}.{}", file_id, file_extension)
        };
        
        let user_dir = disk_path.join("users").join(user_id.to_string());
        let normalized_user_dir = Self::normalize_path(&user_dir)?;
        fs::create_dir_all(&normalized_user_dir)?;

        let file_path = normalized_user_dir.join(&filename);
        
        let mut file = fs::File::create(&file_path)?;
        file.write_all(file_data)?;
        file.sync_all()?;
        
        Ok(StorageResult {
            file_id,
            filename,
            file_path: file_path.to_string_lossy().to_string(),
            disk_path: disk_path.to_string_lossy().to_string(),
            file_size: file_size as i64,
        })
    }
    
    pub fn get_file_data(&self, file_path: &str) -> anyhow::Result<Vec<u8>> {
        let path = PathBuf::from(file_path);
        let normalized_path = Self::normalize_path(&path)?;
        
        if !normalized_path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }
        
        let data = fs::read(&normalized_path)?;
        Ok(data)
    }
    
    pub fn delete_file(&self, file_path: &str) -> anyhow::Result<()> {
        let path = PathBuf::from(file_path);
        let normalized_path = Self::normalize_path(&path)?;
        
        if normalized_path.exists() {
            fs::remove_file(&normalized_path)?;
        }
        
        Ok(())
    }
    
    pub fn file_exists(&self, file_path: &str) -> bool {
        let path = PathBuf::from(file_path);
        if let Ok(normalized_path) = Self::normalize_path(&path) {
            normalized_path.exists()
        } else {
            false
        }
    }

    pub fn create_temp_file(
        &self,
        user_id: &Uuid,
        upload_id: &Uuid,
        total_size: u64,
    ) -> anyhow::Result<(PathBuf, PathBuf)> {
        let disk_path = match self.find_available_disk(total_size)? {
            Some(path) => path,
            None => {
                return Err(anyhow::anyhow!("No available disk space for file"));
            }
        };

        let temp_dir = disk_path.join("temp").join(user_id.to_string());
        let normalized_temp_dir = Self::normalize_path(&temp_dir)?;
        fs::create_dir_all(&normalized_temp_dir)?;

        let temp_file_path = normalized_temp_dir.join(format!("{}.tmp", upload_id));
        
        let file = fs::File::create(&temp_file_path)?;
        file.set_len(total_size)?;
        
        Ok((temp_file_path, disk_path))
    }

    pub fn write_chunk(
        &self,
        temp_file_path: &Path,
        chunk_data: &[u8],
        chunk_number: i32,
        chunk_size: i64,
    ) -> anyhow::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(temp_file_path)?;

        let offset = (chunk_number - 1) as u64 * chunk_size as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(chunk_data)?;
        file.sync_all()?;

        Ok(())
    }

    pub fn finalize_chunked_upload(
        &self,
        temp_file_path: &Path,
        user_id: &Uuid,
        original_filename: &str,
        disk_path: &Path,
    ) -> anyhow::Result<StorageResult> {
        let file_id = Uuid::new_v4();
        let file_extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let filename = if file_extension.is_empty() {
            file_id.to_string()
        } else {
            format!("{}.{}", file_id, file_extension)
        };

        let user_dir = disk_path.join("users").join(user_id.to_string());
        let normalized_user_dir = Self::normalize_path(&user_dir)?;
        fs::create_dir_all(&normalized_user_dir)?;

        let final_file_path = normalized_user_dir.join(&filename);
        
        fs::rename(temp_file_path, &final_file_path)?;
        
        let file_size = fs::metadata(&final_file_path)?.len() as i64;

        let _ = self.cleanup_temp_file(temp_file_path);

        Ok(StorageResult {
            file_id,
            filename,
            file_path: final_file_path.to_string_lossy().to_string(),
            disk_path: disk_path.to_string_lossy().to_string(),
            file_size,
        })
    }

    pub fn cleanup_temp_file(&self, temp_file_path: &Path) -> anyhow::Result<()> {
        if temp_file_path.exists() {
            fs::remove_file(temp_file_path)?;
        }
        Ok(())
    }

    pub fn verify_chunk_integrity(
        &self,
        temp_file_path: &Path,
        chunk_number: i32,
        chunk_size: i64,
        expected_size: usize,
    ) -> anyhow::Result<bool> {
        let mut file = fs::File::open(temp_file_path)?;
        let offset = (chunk_number - 1) as u64 * chunk_size as u64;
        file.seek(SeekFrom::Start(offset))?;

        let mut buffer = vec![0u8; expected_size];
        let bytes_read = file.read(&mut buffer)?;
        
        Ok(bytes_read == expected_size)
    }

    pub fn cleanup_old_temp_files(&self, max_age_hours: u64) -> anyhow::Result<usize> {
        let mut cleaned_count = 0;
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let max_age_seconds = max_age_hours * 3600;

        for storage_path in &self.storage_paths {
            let temp_dir = storage_path.join("temp");
            if !temp_dir.exists() {
                continue;
            }

            cleaned_count += self.cleanup_temp_directory(&temp_dir, current_time, max_age_seconds)?;
        }

        Ok(cleaned_count)
    }

    fn cleanup_temp_directory(&self, temp_dir: &Path, current_time: u64, max_age_seconds: u64) -> anyhow::Result<usize> {
        let mut cleaned_count = 0;

        if let Ok(entries) = fs::read_dir(temp_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        cleaned_count += self.cleanup_temp_directory(&path, current_time, max_age_seconds)?;
                        
                        if let Ok(entries) = fs::read_dir(&path) {
                            if entries.count() == 0 {
                                let _ = fs::remove_dir(&path);
                            }
                        }
                    } else if path.extension().and_then(|s| s.to_str()) == Some("tmp") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(modified_time) = modified.duration_since(UNIX_EPOCH) {
                                    let file_age = current_time.saturating_sub(modified_time.as_secs());
                                    
                                    if file_age > max_age_seconds {
                                        if fs::remove_file(&path).is_ok() {
                                            cleaned_count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    pub fn cleanup_orphaned_temp_files(&self) -> anyhow::Result<usize> {
        self.cleanup_old_temp_files(24)
    }

    pub fn get_temp_files_info(&self) -> anyhow::Result<String> {
        let mut report = String::new();
        let mut total_files = 0;
        let mut total_size = 0u64;
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        for (index, storage_path) in self.storage_paths.iter().enumerate() {
            let temp_dir = storage_path.join("temp");
            if !temp_dir.exists() {
                continue;
            }

            let (files, size) = self.scan_temp_directory(&temp_dir, current_time)?;
            total_files += files;
            total_size += size;

            report.push_str(&format!(
                "Disk {}: {} temp files, {:.2} MB\n",
                index + 1,
                files,
                size as f64 / 1024.0 / 1024.0
            ));
        }

        report.push_str(&format!(
            "\nTotal: {} temp files, {:.2} MB",
            total_files,
            total_size as f64 / 1024.0 / 1024.0
        ));

        Ok(report)
    }

    fn scan_temp_directory(&self, temp_dir: &Path, current_time: u64) -> anyhow::Result<(usize, u64)> {
        let mut file_count = 0;
        let mut total_size = 0u64;

        if let Ok(entries) = fs::read_dir(temp_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        let (sub_files, sub_size) = self.scan_temp_directory(&path, current_time)?;
                        file_count += sub_files;
                        total_size += sub_size;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("tmp") {
                        file_count += 1;
                        if let Ok(metadata) = entry.metadata() {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok((file_count, total_size))
    }
}