# Local Drive

A self-hosted Google Drive alternative built with Rust and Next.js. Store and manage your files across multiple disks with a modern web interface.

## Features

- 🚀 **Self-hosted** - Complete control over your data
- 💾 **Multi-disk support** - Automatically distribute files across multiple storage devices
- 🔐 **Secure authentication** - JWT-based auth with Argon2 password hashing
- 📤 **Chunked uploads** - Support for large file uploads with progress tracking
- 🧹 **Automatic cleanup** - Smart temporary file management with scheduled cleanup
- 🌐 **Modern UI** - Clean, responsive interface built with Next.js and TailwindCSS
- 👥 **User management** - Admin panel for managing users and monitoring storage
- 🔗 **File sharing** - Generate shareable links with permissions
- 🌍 **Multi-language** - Support for English and Turkish
- ⚡ **Real-time updates** - WebSocket support for live upload progress
- 🖼️ **File previews** - Thumbnail generation for images and videos

## Tech Stack

### Backend
- **Rust** with Axum framework
- **PostgreSQL** with SQLx
- **JWT** authentication
- **Tokio** for async operations

### Frontend
- **Next.js 14** (App Router)
- **TypeScript**
- **TailwindCSS**

## Prerequisites

- **Rust** (latest stable)
- **Node.js** 18+ or **Bun**
- **PostgreSQL** 14+
- **Git**

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/sw3do/local-drive.git
cd local-drive
```

### 2. Setup Backend

```bash
cd backend

# Copy environment file
cp .env.example .env

# Edit .env with your configuration
nano .env
```

#### Configure your `.env` file:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/localdrive
STORAGE_PATHS=/path/to/storage1,/path/to/storage2
PORT=3001
JWT_SECRET=your-super-secret-jwt-key-here
```

#### Setup database:

```bash
# Install SQLx CLI
cargo install sqlx-cli

# Create database
createdb localdrive

# Run migrations
sqlx migrate run

# Build and run
cargo run
```

### 3. Setup Frontend

```bash
cd ../frontend

# Copy environment file
cp .env.example .env.local

# Edit .env.local
nano .env.local
```

#### Configure your `.env.local` file:

```env
NEXT_PUBLIC_API_URL=http://localhost:3001
```

#### Install dependencies and run:

```bash
# Using npm
npm install
npm run dev

# Or using bun
bun install
bun dev
```

## Creating Admin User

### Method 1: Using CLI (Recommended)

```bash
cd backend

# Create admin user
cargo run -- create-admin \
  --username admin \
  --email admin@example.com \
  --password your-secure-password
```

### Method 2: Using Database

```sql
# Connect to your PostgreSQL database
psql -d localdrive

# Update user role to admin
UPDATE users SET is_admin = true WHERE email = 'user@example.com';
```

### Method 3: Manual Registration + Database Update

```bash
# 1. First register a user via the web interface
# 2. Then update the user to admin in database
psql -d localdrive -c "UPDATE users SET is_admin = true WHERE email = 'user@example.com';"
```

## Storage Configuration

### Single Disk
```env
STORAGE_PATHS=/home/user/storage
```

### Multiple Disks (Priority Order)
```env
# Linux/macOS
STORAGE_PATHS=/mnt/disk1,/mnt/disk2,/home/backup

# Windows
STORAGE_PATHS=C:\storage,D:\storage,E:\backup
```

Files are automatically distributed across disks when the current disk becomes full.

## API Endpoints

### Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration
- `POST /api/auth/refresh` - Refresh JWT token

### Files
- `GET /api/files` - List user files
- `POST /api/files/upload` - Upload file (chunked)
- `GET /api/files/:id/download` - Download file
- `DELETE /api/files/:id` - Delete file
- `POST /api/files/:id/share` - Create share link

### Admin
- `GET /api/admin/users` - List all users
- `POST /api/admin/users` - Create user
- `DELETE /api/admin/users/:id` - Delete user
- `GET /api/admin/storage` - Storage statistics

## Development

### Backend Development

```bash
cd backend

# Run with auto-reload
cargo watch -x run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Frontend Development

```bash
cd frontend

# Development server
npm run dev

# Type checking
npm run type-check

# Linting
npm run lint

# Build for production
npm run build
```

## Production Deployment

### Using Docker (Recommended)

```bash
# Build and run with docker-compose
docker-compose up -d
```

### Manual Deployment

#### Backend
```bash
cd backend
cargo build --release
./target/release/local-drive
```

#### Frontend
```bash
cd frontend
npm run build
npm start
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name yourdomain.com;

    location /api {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## API Endpoints

### Authentication
- `POST /auth/login` - User login

### File Management
- `GET /files` - List user files
- `GET /files/:id/download` - Download file
- `DELETE /files/:id` - Delete file

### Chunked Upload
- `POST /upload/initiate` - Start chunked upload
- `POST /upload/:upload_id/chunk/:chunk_number` - Upload chunk
- `POST /upload/:upload_id/complete` - Complete upload
- `GET /upload/:upload_id/status` - Get upload status
- `DELETE /upload/:upload_id/cancel` - Cancel upload

### Admin Routes
- `GET /admin/users` - List all users
- `GET /admin/storage` - Get storage information
- `GET /admin/storage/report` - Get detailed disk usage report
- `GET /admin/temp/info` - Get temporary files information
- `POST /admin/temp/cleanup` - Clean orphaned temp files (24h+)
- `POST /admin/temp/cleanup/:hours` - Clean temp files older than specified hours

### Storage Information
- `GET /user/storage` - Get user storage info

## Configuration Options

### Backend Environment Variables

| Variable | Description | Default |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `STORAGE_PATHS` | Comma-separated storage paths | `./storage` |
| `PORT` | Server port | `3001` |
| `JWT_SECRET` | JWT signing secret | Required |
| `MAX_FILE_SIZE` | Maximum file size in bytes | `104857600` (100MB) |
| `CORS_ORIGINS` | Allowed CORS origins | `http://localhost:3000` |
| `LOG_LEVEL` | Logging level | `info` |

### Frontend Environment Variables

| Variable | Description | Default |
|----------|-------------|----------|
| `NEXT_PUBLIC_API_URL` | Backend API URL | Required |
| `NEXT_PUBLIC_MAX_FILE_SIZE` | Max file size for uploads | `104857600` |
| `NEXT_PUBLIC_CHUNK_SIZE` | Upload chunk size | `1048576` |

## Troubleshooting

### Common Issues

1. **Database connection failed**
   - Check PostgreSQL is running
   - Verify DATABASE_URL in .env
   - Ensure database exists

2. **Storage permission denied**
   - Check directory permissions
   - Ensure storage paths exist
   - Verify disk space

3. **Upload fails**
   - Check MAX_FILE_SIZE setting
   - Verify available disk space
   - Check network connectivity

4. **Temporary files accumulating**
   - Automatic cleanup runs every 6 hours
   - Manual cleanup via `/admin/temp/cleanup` endpoint
   - Check temp directory permissions

### Logs

```bash
# Backend logs
cd backend
RUST_LOG=debug cargo run

# Frontend logs
cd frontend
npm run dev
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/sw3do/local-drive/issues) page
2. Create a new issue with detailed information
3. Join our community discussions

## Roadmap

- [ ] Mobile app (React Native)
- [ ] File versioning
- [ ] Advanced search and filtering
- [ ] Backup and sync features
- [ ] Plugin system
- [ ] Advanced admin analytics
- [ ] File encryption at rest
- [ ] Two-factor authentication

---

**Made with ❤️ by [sw3do](https://github.com/sw3do)**