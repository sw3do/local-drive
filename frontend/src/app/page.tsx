'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import { ChunkedFileUpload } from '@/components/ui/chunked-file-upload';
import { useAuthStore } from '@/store/auth';
import { filesApi, StorageInfo, FileInfo } from '@/lib/api';
import {
  Upload,
  Settings,
  User,
  LogOut,
  File,
  Download,
  Trash2,
  Grid3X3,
  List,
  Menu,
  X
} from 'lucide-react';
import Link from 'next/link';

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

export default function Dashboard() {
  const router = useRouter();
  const { user, isAuthenticated, logout, initializeAuth, isLoading: authLoading } = useAuthStore();
  const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [authInitialized, setAuthInitialized] = useState(false);
  const [showChunkedUpload, setShowChunkedUpload] = useState(false);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [sidebarOpen, setSidebarOpen] = useState(false);

  useEffect(() => {
    initializeAuth();
    setAuthInitialized(true);
  }, [initializeAuth]);

  useEffect(() => {
    if (!authInitialized) return;
    
    if (!isAuthenticated && !authLoading) {
      router.push('/login');
      return;
    }

    if (isAuthenticated && !authLoading) {
      const fetchData = async () => {
        try {
          const [storageData, filesData] = await Promise.all([
            filesApi.getUserStorageInfo(),
            filesApi.getFiles()
          ]);
          setStorageInfo(storageData);
          setFiles(filesData);
        } catch (error) {
          console.error('Failed to fetch data:', error);
        } finally {
          setLoading(false);
        }
      };

      fetchData();
    }
  }, [isAuthenticated, authLoading, authInitialized, router]);

  const handleLogout = () => {
    logout();
    router.push('/login');
  };

  const handleDownload = async (fileId: string) => {
    try {
      await filesApi.downloadFile(fileId);
    } catch (error) {
      console.error('Failed to download file:', error);
    }
  };

  const handleDelete = async (fileId: string) => {
    try {
      await filesApi.deleteFile(fileId);
      // Refresh files list
      const filesData = await filesApi.getFiles();
      setFiles(filesData);
      // Refresh storage info
      const storageData = await filesApi.getUserStorageInfo();
      setStorageInfo(storageData);
    } catch (error) {
      console.error('Failed to delete file:', error);
    }
  };

  const refreshData = async () => {
    try {
      const [storageData, filesData] = await Promise.all([
        filesApi.getUserStorageInfo(),
        filesApi.getFiles()
      ]);
      setStorageInfo(storageData);
      setFiles(filesData);
    } catch (error) {
      console.error('Failed to refresh data:', error);
    }
  };

  if (!authInitialized || authLoading || (!isAuthenticated && authLoading) || (isAuthenticated && loading)) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/20">
        <div className="text-center space-y-4">
          <div className="flex justify-center">
            <div className="p-4 bg-primary/10 rounded-full">
              <Spinner size="lg" className="text-primary" />
            </div>
          </div>
          <div>
            <h2 className="text-xl font-semibold text-foreground">Loading Local Drive</h2>
            <p className="text-muted-foreground mt-1">Please wait while we prepare your files...</p>
          </div>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return null;
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="border-b border-border bg-card">
        <div className="flex h-16 items-center px-4 sm:px-6">
          <div className="flex items-center space-x-4">
            <Button
              variant="ghost"
              size="icon"
              className="lg:hidden"
              onClick={() => setSidebarOpen(!sidebarOpen)}
            >
              <Menu className="h-5 w-5" />
            </Button>
            <h1 className="text-lg sm:text-xl font-semibold">Local Drive</h1>
          </div>
          
          <div className="ml-auto flex items-center space-x-2 sm:space-x-4">
            {user?.is_admin && (
              <Link href="/admin">
                <Button variant="ghost" size="icon">
                  <Settings className="h-4 w-4" />
                </Button>
              </Link>
            )}
            
            <div className="hidden sm:flex items-center space-x-2">
              <User className="h-4 w-4" />
              <span className="text-sm">{user?.username}</span>
            </div>
            
            <Button variant="ghost" size="icon" onClick={handleLogout}>
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      <div className="flex relative">
        {sidebarOpen && (
          <div 
            className="fixed inset-0 bg-black/50 z-40 lg:hidden" 
            onClick={() => setSidebarOpen(false)}
          />
        )}
        
        <aside className={`
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'}
          lg:translate-x-0 fixed lg:static inset-y-0 left-0 z-50
          w-64 border-r border-border bg-card p-4 transition-transform duration-200 ease-in-out
          lg:block
        `}>
          <div className="flex items-center justify-between lg:hidden mb-4">
            <h2 className="text-lg font-semibold">Menu</h2>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSidebarOpen(false)}
            >
              <X className="h-5 w-5" />
            </Button>
          </div>
          
          <div className="space-y-2">
            <Button 
              className="w-full justify-start" 
              variant="default"
              onClick={() => {
                setShowChunkedUpload(true);
                setSidebarOpen(false);
              }}
            >
              <Upload className="mr-2 h-4 w-4" />
              Upload Files
            </Button>
          </div>
          
          <div className="mt-8">
            <div className="rounded-lg bg-muted p-4">
              <h3 className="text-sm font-medium">Storage</h3>
              <div className="mt-2">
                <div className="h-2 rounded-full bg-background">
                  <div 
                    className="h-2 rounded-full bg-primary transition-all duration-300" 
                    style={{ width: `${storageInfo?.usage_percentage || 0}%` }}
                  />
                </div>
                <p className="mt-1 text-xs text-muted-foreground">
                  {storageInfo ? (
                    `${formatFileSize(storageInfo.used_space)} of ${formatFileSize(storageInfo.total_space)} used`
                  ) : (
                    'Loading storage info...'
                  )}
                </p>
              </div>
              {storageInfo && storageInfo.disk_count > 1 && (
                <div className="mt-3 pt-3 border-t border-border">
                  <p className="text-xs text-muted-foreground">
                    {storageInfo.disk_count} disks available
                  </p>
                </div>
              )}
            </div>
          </div>
          
          <div className="lg:hidden mt-8 pt-4 border-t border-border">
            <div className="flex items-center space-x-2 text-sm text-muted-foreground">
              <User className="h-4 w-4" />
              <span>{user?.username}</span>
            </div>
          </div>
        </aside>

        <main className="flex-1 p-4 sm:p-6 lg:ml-0">
          <div className="mb-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <div>
                <h2 className="text-xl sm:text-2xl font-semibold">Dosyalarım</h2>
                <p className="text-muted-foreground mt-1 sm:mt-2">
                  {files.length} dosya bulundu
                </p>
              </div>
              <div className="flex items-center space-x-2">
                <Button
                  variant={viewMode === 'grid' ? 'default' : 'outline'}
                  size="icon"
                  onClick={() => setViewMode('grid')}
                >
                  <Grid3X3 className="h-4 w-4" />
                </Button>
                <Button
                  variant={viewMode === 'list' ? 'default' : 'outline'}
                  size="icon"
                  onClick={() => setViewMode('list')}
                >
                  <List className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>

          {files.length === 0 ? (
            <div className="flex h-64 items-center justify-center border-2 border-dashed border-border rounded-lg">
              <div className="text-center px-4">
                <File className="mx-auto h-12 w-12 text-muted-foreground" />
                <h3 className="mt-4 text-lg font-medium">Henüz dosya yok</h3>
                <p className="mt-2 text-muted-foreground text-sm sm:text-base">
                  Dosya yüklemek için {window.innerWidth < 1024 ? 'menüdeki' : 'kenar çubuğundaki'} &quot;Upload Files&quot; butonuna tıklayın
                </p>
                <Button 
                  className="mt-4 lg:hidden" 
                  onClick={() => setShowChunkedUpload(true)}
                >
                  <Upload className="mr-2 h-4 w-4" />
                  Upload Files
                </Button>
              </div>
            </div>
          ) : (
            <div className={viewMode === 'grid' ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-3 sm:gap-4' : 'space-y-2'}>
              {files.map((file) => (
                viewMode === 'grid' ? (
                  <div key={file.id} className="border border-border rounded-lg p-3 sm:p-4 hover:bg-muted/50 transition-colors">
                    <div className="flex items-start justify-between mb-3">
                      <File className="h-6 w-6 sm:h-8 sm:w-8 text-primary flex-shrink-0" />
                      <div className="flex space-x-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7 sm:h-8 sm:w-8"
                          onClick={() => handleDownload(file.id)}
                        >
                          <Download className="h-3 w-3 sm:h-4 sm:w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7 sm:h-8 sm:w-8 text-destructive hover:text-destructive"
                          onClick={() => handleDelete(file.id)}
                        >
                          <Trash2 className="h-3 w-3 sm:h-4 sm:w-4" />
                        </Button>
                      </div>
                    </div>
                    <div>
                      <h3 className="font-medium text-xs sm:text-sm truncate" title={file.original_filename}>
                        {file.original_filename}
                      </h3>
                      <p className="text-xs text-muted-foreground mt-1">
                        {formatFileSize(file.file_size)}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {formatDate(file.created_at)}
                      </p>
                      {file.mime_type && (
                        <p className="text-xs text-muted-foreground truncate">
                          {file.mime_type}
                        </p>
                      )}
                    </div>
                  </div>
                ) : (
                  <div key={file.id} className="flex items-center justify-between p-3 border border-border rounded-lg hover:bg-muted/50 transition-colors">
                    <div className="flex items-center space-x-3 min-w-0 flex-1">
                      <File className="h-4 w-4 sm:h-5 sm:w-5 text-primary flex-shrink-0" />
                      <div className="min-w-0 flex-1">
                        <h3 className="font-medium text-sm truncate" title={file.original_filename}>
                          {file.original_filename}
                        </h3>
                        <div className="flex flex-col sm:flex-row sm:items-center sm:space-x-4 text-xs text-muted-foreground mt-1">
                          <span>{formatFileSize(file.file_size)}</span>
                          <span className="hidden sm:inline">{formatDate(file.created_at)}</span>
                          <span className="sm:hidden">{formatDate(file.created_at).split(',')[0]}</span>
                          {file.mime_type && <span className="hidden lg:inline truncate">{file.mime_type}</span>}
                        </div>
                      </div>
                    </div>
                    <div className="flex space-x-1 flex-shrink-0">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 sm:h-8 sm:w-8"
                        onClick={() => handleDownload(file.id)}
                      >
                        <Download className="h-3 w-3 sm:h-4 sm:w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 sm:h-8 sm:w-8 text-destructive hover:text-destructive"
                        onClick={() => handleDelete(file.id)}
                      >
                        <Trash2 className="h-3 w-3 sm:h-4 sm:w-4" />
                      </Button>
                    </div>
                  </div>
                )
              ))}
            </div>
          )}
        </main>
      </div>

      {showChunkedUpload && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-card p-4 sm:p-6 rounded-lg w-full max-w-[600px] max-h-[80vh] overflow-y-auto border border-border">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">Upload Files</h3>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setShowChunkedUpload(false)}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
            <ChunkedFileUpload
               onUploadComplete={() => {
                 refreshData();
               }}
               multiple={true}
             />
          </div>
        </div>
      )}
    </div>
  );
}