use std::env;
use std::ffi::OsString;
use std::fs::{self};
use std::io::{Result};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[cfg(feature = "temp")]
use tempdir;

use {DirEntry, FileSystem, ReadDir, FileExt, Metadata, Permissions};
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
    type File = fs::File;
    type Permissions = fs::Permissions;
    type Metadata = fs::Metadata;

    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        fs::File::open(path)
    }

    fn create<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        fs::File::create(path)
    }

    fn open_with_options<P: AsRef<Path>>(&self, path: P, options: &crate::OpenOptions) -> Result<Self::File> {
        fs::OpenOptions::new()
            .append(options.append)
            .create(options.create)
            .create_new(options.create_new)
            .read(options.read)
            .truncate(options.truncate)
            .write(options.write)
            .open(path)
    }

    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()> {
        fs::set_permissions(path, perm)
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata> {
        fs::metadata(path)
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

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        fs::canonicalize(path)
    }
}

impl FileExt for fs::File {
    type Metadata = fs::Metadata;

    fn metadata(&self) -> Result<Self::Metadata> {
        self.metadata()
    }

    fn set_len(&self, size: u64) -> Result<()> {
        self.set_len(size)
    }
    fn sync_all(&self) -> Result<()> {
        self.sync_all()
    }
    fn sync_data(&self) -> Result<()> {
        self.sync_data()
    }
}

impl Metadata for fs::Metadata {
    type Permissions = fs::Permissions;

    fn is_dir(&self) -> bool {
        self.is_dir()
    }

    fn is_file(&self) -> bool {
        self.is_file()
    }

    fn len(&self) -> u64 {
        self.len()
    }

    fn permissions(&self) -> Self::Permissions {
        self.permissions()
    }
}

impl Permissions for fs::Permissions {
    fn readonly(&self) -> bool {
        self.readonly()
    }

    fn set_readonly(&mut self, readonly: bool) {
        self.set_readonly(readonly)
    }

    #[cfg(unix)]
    fn mode(&self) -> u32 {
        PermissionsExt::mode(self)
    }

    #[cfg(unix)]
    fn set_mode(&mut self, mode: u32) {
        PermissionsExt::set_mode(self, mode);
    }

    #[cfg(unix)]
    fn from_mode(mode: u32) -> Self {
        PermissionsExt::from_mode(mode)
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

#[cfg(feature = "temp")]
impl TempFileSystem for OsFileSystem {
    type TempDir = OsTempDir;

    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir> {
        tempdir::TempDir::new(prefix.as_ref()).map(OsTempDir)
    }
}
