use std::env;
use std::ffi::OsString;
use std::fs::{self, Permissions};
use std::io::{Read, Result, Write, SeekFrom, Seek};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[cfg(feature = "temp")]
use tempdir;

#[cfg(unix)]
use UnixFileSystem;
use {DirEntry, FileSystem, ReadDir, FileExt, Metadata};
#[cfg(feature = "temp")]
use {TempDir, TempFileSystem};

/// Tracks a temporary directory that will be deleted once the struct goes out of scope.
///
/// This is a wrapper around a [`TempDir`].
///
/// [`TempDir`]: https://doc.rust-lang.org/tempdir/tempdir/struct.TempDir.html
#[cfg(feature = "temp")]
#[derive(Debug)]
pub struct OsTempDir(tempdir::TempDir);

#[cfg(feature = "temp")]
impl TempDir for OsTempDir {
    fn path(&self) -> &Path {
        self.0.path()
    }
}

/// An implementation of `FileSystem` that interacts with the actual operating system's file system.
///
/// This is primarily a wrapper for [`fs`] methods.
///
/// [`fs`]: https://doc.rust-lang.org/std/fs/index.html
#[derive(Clone, Debug, Default)]
pub struct OsFileSystem {}

impl OsFileSystem {
    pub fn new() -> Self {
        OsFileSystem {}
    }
}

impl FileSystem for OsFileSystem {
    type DirEntry = fs::DirEntry;
    type ReadDir = fs::ReadDir;
    type File = OsFile;

    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        OsFile::open(path)
    }

    fn create<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        OsFile::create(path)
    }

    fn open_with_options<P: AsRef<Path>>(&self, path: P, options: &crate::OpenOptions) -> Result<Self::File> {
        OsFile::open_with_options(path, options)
    }

    fn current_dir(&self) -> Result<PathBuf> {
        env::current_dir()
    }

    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        env::set_current_dir(path)
    }

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir()
    }

    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_file()
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::create_dir(path)
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::create_dir_all(path)
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::remove_dir(path)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::remove_dir_all(path)
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir> {
        fs::read_dir(path)
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::remove_file(path)
    }

    fn copy_file<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        fs::copy(from, to).and(Ok(()))
    }

    fn rename<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        fs::rename(from, to)
    }

    fn readonly<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        permissions(path.as_ref()).map(|p| p.readonly())
    }

    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result<()> {
        let mut permissions = permissions(path.as_ref())?;

        permissions.set_readonly(readonly);

        fs::set_permissions(path, permissions)
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        fs::canonicalize(path)
    }
}

#[derive(Debug)]
pub struct OsFile(fs::File);

impl OsFile {
    fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        fs::File::open(path).map(|f| OsFile(f))
    }
    fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        fs::File::create(path).map(|f| OsFile(f))
    }
    fn open_with_options<P: AsRef<Path>>(path: P, options: &crate::OpenOptions) -> Result<Self> {
        fs::OpenOptions::new()
            .append(options.append)
            .create(options.create)
            .create_new(options.create_new)
            .read(options.read)
            .truncate(options.truncate)
            .write(options.write)
            .open(path)
        .map(|f| OsFile(f))
    }
}

impl Read for OsFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Seek for OsFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.0.seek(pos)
    }
}

impl Write for OsFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

impl FileExt for OsFile {
    type Metadata = OsMetadata;

    fn metadata(&self) -> Result<Self::Metadata> {
        self.0.metadata().map(|m| OsMetadata(m))
    }

    fn set_len(&self, size: u64) -> Result<()> {
        self.0.set_len(size)
    }
    fn sync_all(&self) -> Result<()> {
        self.0.sync_all()
    }
    fn sync_data(&self) -> Result<()> {
        self.0.sync_data()
    }
}

pub struct OsMetadata(fs::Metadata);

impl Metadata for OsMetadata {
    fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    fn is_file(&self) -> bool {
        self.0.is_file()
    }

    fn len(&self) -> u64 {
        self.0.len()
    }
}

impl DirEntry for fs::DirEntry {
    fn file_name(&self) -> OsString {
        self.file_name()
    }

    fn path(&self) -> PathBuf {
        self.path()
    }
}

impl ReadDir<fs::DirEntry> for fs::ReadDir {}

#[cfg(unix)]
impl UnixFileSystem for OsFileSystem {
    fn mode<P: AsRef<Path>>(&self, path: P) -> Result<u32> {
        permissions(path.as_ref()).map(|p| p.mode())
    }

    fn set_mode<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result<()> {
        let mut permissions = permissions(path.as_ref())?;

        permissions.set_mode(mode);

        fs::set_permissions(path, permissions)
    }
}

#[cfg(feature = "temp")]
impl TempFileSystem for OsFileSystem {
    type TempDir = OsTempDir;

    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir> {
        tempdir::TempDir::new(prefix.as_ref()).map(OsTempDir)
    }
}

fn permissions(path: &Path) -> Result<Permissions> {
    let metadata = fs::metadata(path)?;

    Ok(metadata.permissions())
}
