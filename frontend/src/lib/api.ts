import axios from 'axios';
import { useAuthStore } from '@/store/auth';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true,
});

api.interceptors.request.use((config) => {
  const state = useAuthStore.getState();
  if (state.token) {
    config.headers.Authorization = `Bearer ${state.token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      const { logout } = useAuthStore.getState();
      logout();
      if (typeof window !== 'undefined') {
        window.location.href = '/login';
      }
    }
    return Promise.reject(error);
  }
);

export interface LoginRequest {
  username: string;
  password: string;
}



export interface User {
  id: string;
  username: string;
  email: string;
  password_hash: string;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
}

export interface FileInfo {
  id: string;
  user_id: string;
  filename: string;
  original_filename: string;
  file_path: string;
  disk_path: string;
  file_size: number;
  mime_type?: string;
  created_at: string;
  updated_at: string;
  is_deleted: boolean;
  deleted_at?: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface DiskInfo {
  path: string;
  total_space: number;
  used_space: number;
  available_space: number;
  usage_percentage: number;
  is_accessible: boolean;
}

export interface StorageInfo {
  total_space: number;
  used_space: number;
  available_space: number;
  usage_percentage: number;
  disk_count: number;
  disks: DiskInfo[];
}

export interface InitiateChunkedUploadRequest {
  filename: string;
  total_size: number;
  chunk_size: number;
}

export interface InitiateChunkedUploadResponse {
  upload_id: string;
  chunk_size: number;
  total_chunks: number;
}

export interface UploadChunkResponse {
  chunk_number: number;
  uploaded: boolean;
  upload_completed: boolean;
  file_info?: FileInfo;
}

export interface ChunkedUpload {
  id: string;
  user_id: string;
  filename: string;
  total_size: number;
  chunk_size: number;
  total_chunks: number;
  uploaded_chunks: number;
  temp_path: string;
  disk_path: string;
  is_completed: boolean;
  created_at: string;
  updated_at: string;
}

export interface ChunkedUploadProgress {
  uploadId: string;
  filename: string;
  totalSize: number;
  uploadedSize: number;
  progress: number;
  status: 'uploading' | 'completed' | 'error' | 'cancelled';
  error?: string;
}

export const authApi = {
  login: async (data: LoginRequest): Promise<AuthResponse> => {
    const response = await api.post('/auth/login', data);
    return response.data;
  },


};

export const filesApi = {
  getFiles: async (): Promise<FileInfo[]> => {
    const response = await api.get('/files');
    return response.data;
  },

  uploadFile: async (file: File, onProgress?: (progress: number) => void): Promise<FileInfo> => {
    const formData = new FormData();
    formData.append('file', file);

    const response = await api.post('/files/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (progressEvent) => {
        if (progressEvent.total && onProgress) {
          const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
          onProgress(progress);
        }
      },
    });
    return response.data;
  },

  downloadFile: async (id: string) => {
    const response = await api.get(`/files/${id}/download`, {
      responseType: 'blob',
    });

    const url = window.URL.createObjectURL(new Blob([response.data]));
    const link = document.createElement('a');
    link.href = url;

    const contentDisposition = response.headers['content-disposition'];
    console.log('Content-Disposition header:', contentDisposition);
    console.log('All headers:', response.headers);
    let filename = 'download.txt';
    if (contentDisposition) {
      const filenameMatch = contentDisposition.match(/filename="([^"]+)"/); 
      if (filenameMatch) {
        filename = decodeURIComponent(filenameMatch[1]);
      } else {
        const filenameStarMatch = contentDisposition.match(/filename\*=UTF-8''([^;]+)/);
        if (filenameStarMatch) {
          filename = decodeURIComponent(filenameStarMatch[1]);
        }
      }
    }

    link.setAttribute('download', filename);
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.URL.revokeObjectURL(url);
  },

  deleteFile: async (id: string) => {
    const response = await api.delete(`/files/${id}`);
    return response.data;
  },

  // Trash bin functions
  getTrashFiles: async (): Promise<FileInfo[]> => {
    const response = await api.get('/trash');
    return response.data;
  },

  restoreFile: async (id: string) => {
    const response = await api.post(`/trash/${id}/restore`);
    return response.data;
  },

  deleteFilePermanently: async (id: string) => {
    const response = await api.delete(`/trash/${id}`);
    return response.data;
  },

  getUserStorageInfo: async (): Promise<StorageInfo> => {
    const response = await api.get('/user/storage');
    return response.data;
  },



  shareFile: async (fileId: string, permissions: 'read' | 'write' = 'read'): Promise<{ share_link: string }> => {
    const response = await api.post(`/files/${fileId}/share`, { permissions });
    return response.data;
  },

  initiateChunkedUpload: async (data: InitiateChunkedUploadRequest): Promise<InitiateChunkedUploadResponse> => {
    const response = await api.post('/upload/initiate', data);
    return response.data;
  },

  uploadChunk: async (
    uploadId: string,
    chunkNumber: number,
    chunkData: Blob,
    onProgress?: (progress: number) => void
  ): Promise<UploadChunkResponse> => {
    const response = await api.post(`/upload/${uploadId}/chunk/${chunkNumber}`, chunkData, {
      headers: {
        'Content-Type': 'application/octet-stream',
      },
      onUploadProgress: (progressEvent) => {
        if (onProgress && progressEvent.total) {
          const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
          onProgress(progress);
        }
      },
    });
    return response.data;
  },

  completeChunkedUpload: async (uploadId: string): Promise<FileInfo> => {
    const response = await api.post(`/upload/${uploadId}/complete`);
    return response.data;
  },

  getUploadStatus: async (uploadId: string): Promise<ChunkedUpload> => {
    const response = await api.get(`/upload/${uploadId}/status`);
    return response.data;
  },

  cancelChunkedUpload: async (uploadId: string): Promise<void> => {
    await api.delete(`/upload/${uploadId}/cancel`);
  },

  uploadFileChunked: async (
    file: File,
    chunkSize: number = 1024 * 1024,
    onProgress?: (progress: ChunkedUploadProgress) => void
  ): Promise<FileInfo> => {
    try {
      const initResponse = await filesApi.initiateChunkedUpload({
        filename: file.name,
        total_size: file.size,
        chunk_size: chunkSize,
      });

      const { upload_id, total_chunks } = initResponse;
      let uploadedSize = 0;

      for (let chunkNumber = 1; chunkNumber <= total_chunks; chunkNumber++) {
        const start = (chunkNumber - 1) * chunkSize;
        const end = Math.min(start + chunkSize, file.size);
        const chunk = file.slice(start, end);

        await filesApi.uploadChunk(upload_id, chunkNumber, chunk, (chunkProgress) => {
          const currentChunkSize = end - start;
          const chunkUploadedSize = (chunkProgress / 100) * currentChunkSize;
          const totalUploadedSize = uploadedSize + chunkUploadedSize;
          const totalProgress = (totalUploadedSize / file.size) * 100;

          onProgress?.({
            uploadId: upload_id,
            filename: file.name,
            totalSize: file.size,
            uploadedSize: totalUploadedSize,
            progress: totalProgress,
            status: 'uploading',
          });
        });

        uploadedSize += (end - start);

        onProgress?.({
          uploadId: upload_id,
          filename: file.name,
          totalSize: file.size,
          uploadedSize,
          progress: (uploadedSize / file.size) * 100,
          status: 'uploading',
        });
      }

      const fileInfo = await filesApi.completeChunkedUpload(upload_id);

      onProgress?.({
        uploadId: upload_id,
        filename: file.name,
        totalSize: file.size,
        uploadedSize: file.size,
        progress: 100,
        status: 'completed',
      });

      return fileInfo;
    } catch (error) {
      onProgress?.({
        uploadId: '',
        filename: file.name,
        totalSize: file.size,
        uploadedSize: 0,
        progress: 0,
        status: 'error',
        error: error instanceof Error ? error.message : 'Upload failed',
      });
      throw error;
    }
  },
};

export interface TempFilesInfo {
  total_files: number;
  total_size: number;
  oldest_file_age_hours?: number;
}

export interface CleanupResult {
  cleaned_files: number;
  freed_space: number;
}

export const adminApi = {
  getUsers: async (): Promise<User[]> => {
    const response = await api.get('/admin/users');
    return response.data;
  },

  getStorageInfo: async (): Promise<StorageInfo> => {
    const response = await api.get('/admin/storage');
    return response.data;
  },

  getTempFilesInfo: async (): Promise<TempFilesInfo> => {
    const response = await api.get('/admin/temp/info');
    return response.data;
  },

  cleanupTempFiles: async (): Promise<CleanupResult> => {
    const response = await api.post('/admin/temp/cleanup');
    return response.data;
  },

  cleanupTempFilesWithAge: async (hours: number): Promise<CleanupResult> => {
    const response = await api.post(`/admin/temp/cleanup/${hours}`);
    return response.data;
  },
};

export default api;