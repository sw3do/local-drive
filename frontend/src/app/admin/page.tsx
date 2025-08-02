'use client';

import { useEffect, useState } from 'react';
import { useAuthStore } from '@/store/auth';
import { adminApi, User, StorageInfo, TempFilesInfo } from '@/lib/api';
import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { ArrowLeft } from 'lucide-react';



export default function AdminPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
  const [tempFilesInfo, setTempFilesInfo] = useState<TempFilesInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [cleanupLoading, setCleanupLoading] = useState(false);
  const [customHours, setCustomHours] = useState<string>('24');
  const { user, isAuthenticated, initializeAuth, isLoading: authLoading } = useAuthStore();
  const router = useRouter();
  const [authInitialized, setAuthInitialized] = useState(false);

  useEffect(() => {
    initializeAuth();
    setAuthInitialized(true);
  }, [initializeAuth]);

  useEffect(() => {
    if (!authInitialized) return;
    
    if ((!isAuthenticated || !user?.is_admin) && !authLoading) {
      router.push('/login');
      return;
    }

    if (isAuthenticated && user?.is_admin && !authLoading) {
      const fetchData = async () => {
        try {
          const [usersData, storageData, tempData] = await Promise.all([
            adminApi.getUsers(),
            adminApi.getStorageInfo(),
            adminApi.getTempFilesInfo(),
          ]);
          setUsers(usersData);
          setStorageInfo(storageData);
          setTempFilesInfo(tempData);
        } catch (error) {
          console.error('Failed to fetch admin data:', error);
        } finally {
          setLoading(false);
        }
      };

      fetchData();
    }
  }, [isAuthenticated, user, authLoading, authInitialized, router]);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const handleCleanupTempFiles = async (hours?: number) => {
    setCleanupLoading(true);
    try {
      let result;
      if (hours) {
        result = await adminApi.cleanupTempFilesWithAge(hours);
      } else {
        result = await adminApi.cleanupTempFiles();
      }
      
      const sizeInMB = (result.freed_space / (1024 * 1024)).toFixed(2);
      alert(`Cleanup completed: ${result.cleaned_files} files removed, ${sizeInMB} MB freed`);
      
      // Refresh temp files info
      const tempInfo = await adminApi.getTempFilesInfo();
      setTempFilesInfo(tempInfo);
    } catch (error) {
      console.error('Cleanup failed:', error);
      alert('Cleanup failed');
    } finally {
      setCleanupLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background via-background to-muted/20">
        <div className="text-center space-y-4">
          <div className="flex justify-center">
            <div className="p-4 bg-primary/10 rounded-full">
              <Spinner size="lg" className="text-primary" />
            </div>
          </div>
          <div>
            <h2 className="text-xl font-semibold text-foreground">Loading Admin Panel</h2>
            <p className="text-muted-foreground mt-1">Preparing administrative data...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      <div className="border-b border-border bg-card">
        <div className="flex h-16 items-center px-4 sm:px-6">
          <div className="flex items-center space-x-4">
            <Link href="/">
              <Button variant="ghost" size="icon">
                <ArrowLeft className="h-4 w-4" />
              </Button>
            </Link>
            <h1 className="text-lg sm:text-xl font-semibold">Admin Panel</h1>
          </div>
        </div>
      </div>

      <div className="p-4 sm:p-6">
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-4 sm:gap-6">
        <div className="bg-card rounded-lg border border-border p-4 sm:p-6">
          <h2 className="text-lg sm:text-xl font-semibold mb-4">User Management</h2>
          <div className="space-y-3">
            {users.map((user) => (
              <div key={user.id} className="flex flex-col sm:flex-row sm:items-center p-3 sm:p-4 border border-border rounded gap-3 sm:gap-0">
                <div className="flex-1">
                  <div className="font-medium">{user.username}</div>
                  <div className="text-sm text-muted-foreground">{user.email}</div>
                  <div className="text-xs text-muted-foreground mt-1">
                    Created: {new Date(user.created_at).toLocaleDateString()}
                  </div>
                  {user.is_admin && (
                    <span className="inline-block bg-blue-100 text-blue-800 text-xs px-2 py-1 rounded mt-1">
                      Admin
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="bg-card rounded-lg border border-border p-4 sm:p-6">
          <h2 className="text-lg sm:text-xl font-semibold mb-4">Storage Usage</h2>
          {storageInfo && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                <div className="bg-muted p-3 sm:p-4 rounded">
                  <div className="text-sm text-muted-foreground">Total Storage</div>
                  <div className="text-lg sm:text-2xl font-bold">{formatBytes(storageInfo.total_space)}</div>
                </div>
                <div className="bg-muted p-3 sm:p-4 rounded">
                  <div className="text-sm text-muted-foreground">Used Storage</div>
                  <div className="text-lg sm:text-2xl font-bold">{formatBytes(storageInfo.used_space)}</div>
                </div>
                <div className="bg-muted p-3 sm:p-4 rounded">
                  <div className="text-sm text-muted-foreground">Available Storage</div>
                  <div className="text-lg sm:text-2xl font-bold">{formatBytes(storageInfo.available_space)}</div>
                </div>
              </div>
              
              <div className="bg-muted p-3 sm:p-4 rounded">
                <div className="text-sm text-muted-foreground mb-2">Overall Usage</div>
                <div className="w-full bg-background rounded-full h-3">
                  <div 
                    className="bg-primary h-3 rounded-full transition-all duration-300" 
                    style={{ width: `${storageInfo.usage_percentage}%` }}
                  ></div>
                </div>
                <div className="text-sm text-muted-foreground mt-1">
                  {storageInfo.usage_percentage}% used
                </div>
              </div>
              
              <div className="mt-6">
                <h3 className="text-base sm:text-lg font-medium mb-3">Disk Details ({storageInfo.disk_count} disks)</h3>
                <div className="space-y-3">
                  {storageInfo.disks.map((disk, index) => (
                    <div key={index} className="border border-border rounded p-3 sm:p-4">
                      <div className="flex flex-col sm:flex-row sm:justify-between sm:items-center mb-2 gap-2">
                        <span className="font-medium font-mono text-sm truncate">{disk.path}</span>
                        <span className={`px-2 py-1 rounded text-xs self-start ${
                          disk.is_accessible ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                        }`}>
                          {disk.is_accessible ? 'Accessible' : 'Inaccessible'}
                        </span>
                      </div>
                      <div className="text-sm text-muted-foreground mb-2">
                        {formatBytes(disk.used_space)} / {formatBytes(disk.total_space)} ({disk.usage_percentage}%)
                      </div>
                      <div className="w-full bg-muted rounded-full h-2">
                        <div 
                          className={`h-2 rounded-full transition-all duration-300 ${
                            disk.usage_percentage > 90 ? 'bg-red-500' : 
                            disk.usage_percentage > 75 ? 'bg-yellow-500' : 'bg-green-500'
                          }`}
                          style={{ width: `${disk.usage_percentage}%` }}
                        ></div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
        
        {/* Temporary Files Management */}
        <div className="bg-card rounded-lg border border-border p-4 sm:p-6 xl:col-span-2">
          <h2 className="text-lg sm:text-xl font-semibold mb-4">Temporary Files Management</h2>
          {tempFilesInfo && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <div className="bg-muted p-3 sm:p-4 rounded">
                  <div className="text-sm text-muted-foreground">Total Temp Files</div>
                  <div className="text-lg sm:text-2xl font-bold">{tempFilesInfo.total_files}</div>
                </div>
                <div className="bg-muted p-3 sm:p-4 rounded">
                  <div className="text-sm text-muted-foreground">Total Size</div>
                  <div className="text-lg sm:text-2xl font-bold">{formatBytes(tempFilesInfo.total_size)}</div>
                </div>
                <div className="bg-muted p-3 sm:p-4 rounded">
                  <div className="text-sm text-muted-foreground">Oldest File Age</div>
                  <div className="text-lg sm:text-2xl font-bold">
                    {tempFilesInfo.oldest_file_age_hours ? `${tempFilesInfo.oldest_file_age_hours.toFixed(1)}h` : 'N/A'}
                  </div>
                </div>
              </div>
              
              <div className="border-t border-border pt-4">
                <h3 className="text-base sm:text-lg font-medium mb-3">Cleanup Actions</h3>
                <div className="flex flex-col sm:flex-row gap-3">
                  <Button 
                    onClick={() => handleCleanupTempFiles()}
                    disabled={cleanupLoading}
                    className="flex-1"
                  >
                    {cleanupLoading ? (
                      <>
                        <Spinner size="sm" className="mr-2" />
                        Cleaning...
                      </>
                    ) : (
                      'Cleanup Old Files (24h+)'
                    )}
                  </Button>
                  
                  <div className="flex flex-col sm:flex-row gap-2 flex-1">
                    <input
                      type="number"
                      value={customHours}
                      onChange={(e) => setCustomHours(e.target.value)}
                      placeholder="Hours"
                      min="1"
                      className="px-3 py-2 border border-border rounded text-sm flex-1"
                    />
                    <Button 
                      onClick={() => handleCleanupTempFiles(parseInt(customHours))}
                      disabled={cleanupLoading || !customHours || parseInt(customHours) < 1}
                      variant="outline"
                      className="flex-1"
                    >
                      Custom Cleanup
                    </Button>
                  </div>
                </div>
                
                <div className="mt-3 text-xs text-muted-foreground">
                  <p>• Automatic cleanup runs every 6 hours for files older than 24 hours</p>
                  <p>• Manual cleanup allows you to specify custom age threshold</p>
                  <p>• Only temporary upload files (.tmp) are affected</p>
                </div>
              </div>
            </div>
          )}
        </div>
        </div>
      </div>
    </div>
  );
}