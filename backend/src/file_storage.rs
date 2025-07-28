use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read, Seek, SeekFrom};
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
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
            storage_paths.push(path);
        }
        
        Ok(FileStorage { storage_paths })
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
        let metadata = fs::metadata(path)?;
        
        let (total_space, available_space) = if cfg!(target_os = "windows") {
            self.get_windows_disk_space(path)?
        } else {
            self.get_unix_disk_space(path)?
        };
        
        let used_space = total_space - available_space;
        let usage_percentage = if total_space > 0 {
            (used_space as f64 / total_space as f64 * 100.0) as u8
        } else {
            0
        };
        
        Ok(DiskInfo {
            path: path.to_string_lossy().to_string(),
            total_space,
            used_space,
            available_space,
            usage_percentage,
            is_accessible: path.exists() && metadata.is_dir(),
        })
    }
    
    #[cfg(target_os = "windows")]
    fn get_windows_disk_space(&self, path: &Path) -> anyhow::Result<(u64, u64)> {
        use std::ffi::CString;
        use std::mem;
        use winapi::um::fileapi::GetDiskFreeSpaceExA;
        
        let path_cstring = CString::new(path.to_string_lossy().as_bytes())?;
        let mut free_bytes_available = 0u64;
        let mut total_number_of_bytes = 0u64;
        let mut total_number_of_free_bytes = 0u64;
        
        unsafe {
            let result = GetDiskFreeSpaceExA(
                path_cstring.as_ptr(),
                &mut free_bytes_available,
                &mut total_number_of_bytes,
                &mut total_number_of_free_bytes,
            );
            
            if result == 0 {
                return Err(anyhow::anyhow!("Failed to get disk space information"));
            }
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
    
    pub fn find_available_disk(&self, file_size: u64) -> anyhow::Result<Option<PathBuf>> {
        for path in &self.storage_paths {
            let disk_info = self.get_single_disk_info(path, 0)?;
            
            if disk_info.is_accessible && disk_info.available_space > file_size {
                return Ok(Some(path.clone()));
            }
        }
        
        Ok(None)
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
        fs::create_dir_all(&user_dir)?;
        
        let file_path = user_dir.join(&filename);
        
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
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found"));
        }
        
        let data = fs::read(path)?;
        Ok(data)
    }
    
    pub fn delete_file(&self, file_path: &str) -> anyhow::Result<()> {
        let path = Path::new(file_path);
        
        if path.exists() {
            fs::remove_file(path)?;
        }
        
        Ok(())
    }
    
    pub fn file_exists(&self, file_path: &str) -> bool {
        Path::new(file_path).exists()
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
        fs::create_dir_all(&temp_dir)?;

        let temp_file_path = temp_dir.join(format!("{}.tmp", upload_id));
        
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
        fs::create_dir_all(&user_dir)?;

        let final_file_path = user_dir.join(&filename);
        
        fs::rename(temp_file_path, &final_file_path)?;
        
        let file_size = fs::metadata(&final_file_path)?.len() as i64;

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
}