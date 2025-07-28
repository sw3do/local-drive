'use client';

import React, { useState, useRef } from 'react';
import { Button } from './button';
import { filesApi, ChunkedUploadProgress } from '@/lib/api';

interface ChunkedFileUploadProps {
  onUploadComplete?: (fileInfo: unknown) => void;
  onUploadProgress?: (progress: ChunkedUploadProgress) => void;
  chunkSize?: number;
  maxFileSize?: number;
  acceptedFileTypes?: string[];
  multiple?: boolean;
  className?: string;
}

interface UploadItem {
  id: string;
  file: File;
  progress: ChunkedUploadProgress;
}

export function ChunkedFileUpload({
  onUploadComplete,
  onUploadProgress,
  chunkSize = 1024 * 1024,
  maxFileSize = Infinity,
  acceptedFileTypes = [],
  multiple = false,
  className = '',
}: ChunkedFileUploadProps) {
  const [uploads, setUploads] = useState<UploadItem[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const validateFile = (file: File): string | null => {
    if (file.size > maxFileSize) {
      return `File size exceeds ${Math.round(maxFileSize / (1024 * 1024))}MB limit`;
    }

    if (acceptedFileTypes.length > 0) {
      const fileExtension = file.name.split('.').pop()?.toLowerCase();
      const mimeType = file.type;
      
      const isValidType = acceptedFileTypes.some(type => {
        if (type.startsWith('.')) {
          return type.slice(1) === fileExtension;
        }
        return type === mimeType || type === `${mimeType.split('/')[0]}/*`;
      });

      if (!isValidType) {
        return `File type not supported. Accepted types: ${acceptedFileTypes.join(', ')}`;
      }
    }

    return null;
  };

  const handleFileSelect = (files: FileList) => {
    const fileArray = Array.from(files);
    
    if (!multiple && fileArray.length > 1) {
      alert('Only one file can be uploaded at a time');
      return;
    }

    const validFiles: File[] = [];
    const errors: string[] = [];

    fileArray.forEach(file => {
      const error = validateFile(file);
      if (error) {
        errors.push(`${file.name}: ${error}`);
      } else {
        validFiles.push(file);
      }
    });

    if (errors.length > 0) {
      alert(`Upload errors:\n${errors.join('\n')}`);
    }

    validFiles.forEach(file => {
      uploadFile(file);
    });
  };

  const uploadFile = async (file: File) => {
    const uploadId = Math.random().toString(36).substr(2, 9);
    const initialProgress: ChunkedUploadProgress = {
      uploadId,
      filename: file.name,
      totalSize: file.size,
      uploadedSize: 0,
      progress: 0,
      status: 'uploading',
    };

    const uploadItem: UploadItem = {
      id: uploadId,
      file,
      progress: initialProgress,
    };

    setUploads(prev => [...prev, uploadItem]);

    try {
      const fileInfo = await filesApi.uploadFileChunked(
        file,
        chunkSize,
        (progress) => {
          setUploads(prev => 
            prev.map(item => 
              item.id === uploadId 
                ? { ...item, progress }
                : item
            )
          );
          onUploadProgress?.(progress);
        }
      );

      setUploads(prev => prev.filter(item => item.id !== uploadId));
      setSuccessMessage(`${file.name} başarıyla yüklendi!`);
      setTimeout(() => setSuccessMessage(null), 3000);
      onUploadComplete?.(fileInfo);
    } catch (error) {
      setUploads(prev => 
        prev.map(item => 
          item.id === uploadId 
            ? { 
                ...item, 
                progress: { 
                  ...item.progress, 
                  status: 'error', 
                  error: error instanceof Error ? error.message : 'Upload failed' 
                }
              }
            : item
        )
      );
    }
  };

  const cancelUpload = async (uploadId: string) => {
    const upload = uploads.find(u => u.id === uploadId);
    if (upload?.progress.uploadId) {
      try {
        await filesApi.cancelChunkedUpload(upload.progress.uploadId);
      } catch (error) {
        console.error('Failed to cancel upload:', error);
      }
    }
    setUploads(prev => prev.filter(item => item.id !== uploadId));
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      handleFileSelect(files);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };



  return (
    <div className={`space-y-4 ${className}`}>
      {successMessage && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-3 sm:p-4">
          <div className="flex items-center">
            <svg className="h-5 w-5 text-green-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
            </svg>
            <p className="text-sm font-medium text-green-800">{successMessage}</p>
          </div>
        </div>
      )}
      <div
        className={`border-2 border-dashed rounded-lg p-4 sm:p-6 lg:p-8 text-center transition-colors ${
          isDragOver 
            ? 'border-blue-500 bg-blue-50' 
            : 'border-gray-300 hover:border-gray-400'
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="space-y-3 sm:space-y-4">
          <div className="text-gray-600">
            <svg className="mx-auto h-8 w-8 sm:h-10 sm:w-10 lg:h-12 lg:w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
              <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </div>
          <div>
            <p className="text-base sm:text-lg font-medium text-gray-900">
              <span className="hidden sm:inline">Drag and drop files here, or{' '}</span>
              <button
                type="button"
                className="text-blue-600 hover:text-blue-500 font-medium"
                onClick={() => fileInputRef.current?.click()}
              >
                <span className="sm:hidden">Tap to select files</span>
                <span className="hidden sm:inline">browse</span>
              </button>
            </p>
            <p className="text-xs sm:text-sm text-gray-500 mt-1 px-2">
               <span className="block sm:inline">{multiple ? 'Multiple files supported' : 'Single file only'}</span>
               {acceptedFileTypes.length > 0 && (
                 <>
                   <span className="hidden sm:inline"> • </span>
                   <span className="block sm:inline">Accepted: {acceptedFileTypes.join(', ')}</span>
                 </>
               )}
             </p>
          </div>
          <Button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            className="mx-auto w-full sm:w-auto"
            size="sm"
          >
            Select Files
          </Button>
        </div>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        multiple={multiple}
        accept={acceptedFileTypes.join(',')}
        onChange={(e) => {
          if (e.target.files) {
            handleFileSelect(e.target.files);
            e.target.value = '';
          }
        }}
        className="hidden"
      />

      {uploads.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-base sm:text-lg font-medium text-gray-900">Uploading Files</h3>
          {uploads.map((upload) => (
            <div key={upload.id} className="bg-white border rounded-lg p-3 sm:p-4 shadow-sm">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between mb-2 gap-2">
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-gray-900 truncate">
                    {upload.file.name}
                  </p>
                  <p className="text-xs text-gray-500">
                    {formatFileSize(upload.file.size)}
                  </p>
                </div>
                <div className="flex items-center justify-between sm:justify-end space-x-2">
                  <span className="text-sm text-gray-600">
                    {upload.progress.progress.toFixed(1)}%
                  </span>
                  {upload.progress.status === 'uploading' && (
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => cancelUpload(upload.id)}
                    >
                      Cancel
                    </Button>
                  )}
                </div>
              </div>
              
              <div className="w-full bg-gray-200 rounded-full h-2 mb-2">
                <div
                  className={`h-2 rounded-full transition-all duration-300 ${
                    upload.progress.status === 'error' 
                      ? 'bg-red-500' 
                      : upload.progress.status === 'completed'
                      ? 'bg-green-500'
                      : 'bg-blue-500'
                  }`}
                  style={{ width: `${upload.progress.progress}%` }}
                />
              </div>
              
              <div className="flex flex-col sm:flex-row sm:justify-between text-xs text-gray-500 gap-1">
                <span>
                  {formatFileSize(upload.progress.uploadedSize)} / {formatFileSize(upload.progress.totalSize)}
                </span>
                <span className="capitalize">
                  {upload.progress.status}
                  {upload.progress.error && `: ${upload.progress.error}`}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}