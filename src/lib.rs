#![warn(clippy::all)]

use std::ffi::OsString;
use std::io::{self, Result};
use std::fmt;
use std::path::{Path, PathBuf};

#[cfg(feature = "fake")]
pub use fake::{FakeFileSystem};
pub use os::OsFileSystem;
#[cfg(feature = "temp")]
pub use os::OsTempDir;

#[cfg(feature = "fake")]
mod fake;
mod os;

/// Provides standard file system operations.
pub trait FileSystem: Clone + Send + Sync {
    type DirEntry: DirEntry;
    type ReadDir: ReadDir<Self::DirEntry>;
    type File: io::Read + io::Seek + io::Write + FileExt<Metadata=Self::Metadata> + fmt::Debug;
    type Permissions: Permissions;
    type Metadata: Metadata<Permissions=Self::Permissions>;

    /// Attempts to open a file in read-only mode.
    /// This is based on [`fs::File::open`].
    ///
    /// [`fs::File::open`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.open
    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File>;

    /// Opens a file in write-only mode.
    /// This function will create a file if it does not exist, and will truncate it if it does.
    /// This is based on [`fs::File::create`].
    ///
    /// [`fs::File::create`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.create
    fn create<P: AsRef<Path>>(&self, path: P) -> Result<Self::File>;

    /// Opens a file at path with the options specified by self.
    /// This is based on [`fs::OpenOptions::open`].
    ///
    /// On the FakeFileSystem, currently the only supported options are:
    /// - `new().create(true).write(true).truncate(true)`, equivalent to `File::create`
    /// - `new().read(true)`, equivalent to `File::open`
    /// - `new().write(true)`
    ///
    /// [`fs::OpenOptions::open`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.open
    fn open_with_options<P: AsRef<Path>>(&self, path: P, options: &OpenOptions) -> Result<Self::File>;

    /// Changes the permissions found on a file or a directory.
    /// This is based on [`fs::set_permissions`].
    ///
    /// [`fs::set_permissions`]: https://doc.rust-lang.org/std/fs/fn.set_permissions.html
    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()>;

    /// Given a path, query the file system to get information about a file, directory, etc.
    /// This is based on [`fs::metadata`].
    ///
    /// [`fs::metadata`]: https://doc.rust-lang.org/std/fs/fn.metadata.html
    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata>;

    /// Returns the current working directory.
    /// This is based on [`std::env::current_dir`].
    ///
    /// [`std::env::current_dir`]: https://doc.rust-lang.org/std/env/fn.current_dir.html
    fn current_dir(&self) -> Result<PathBuf>;
    /// Updates the current working directory.
    /// This is based on [`std::env::set_current_dir`].
    ///
    /// [`std::env::set_current_dir`]: https://doc.rust-lang.org/std/env/fn.set_current_dir.html
    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Returns true if the path exists on disk and is pointing at a directory.
    /// This is based on [`std::path::Path::is_dir`]
    ///
    /// [`std::path::Path::is_dir`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.is_dir
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool;

    /// Returns true if the path exists on disk and is pointing at a regular file.
    /// This is based on [`std::path::Path::is_file`]
    ///
    /// [`std::path::Path::is_file`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.is_file
    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool;

    /// Creates a new directory.
    /// This is based on [`std::fs::create_dir`].
    ///
    /// [`std::fs::create_dir`]: https://doc.rust-lang.org/std/fs/fn.create_dir.html
    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// Recursively creates a directory and any missing parents.
    /// This is based on [`std::fs::create_dir`].
    ///
    /// [`std::fs::create_dir_all`]: https://doc.rust-lang.org/std/fs/fn.create_dir_all.html
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// Removes an empty directory.
    /// This is based on [`std::fs::remove_dir`].
    ///
    /// [`std::fs::remove_dir`]: https://doc.rust-lang.org/std/fs/fn.remove_dir.html
    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// Removes a directory and any child files or directories.
    /// This is based on [`std::fs::remove_dir_all`].
    ///
    /// [`std::fs::remove_dir_all`]: https://doc.rust-lang.org/std/fs/fn.remove_dir_all.html
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// Returns an iterator over the entries in a directory.
    /// This is based on [`std::fs::read_dir`].
    ///
    /// [`std::fs::read_dir`]: https://doc.rust-lang.org/std/fs/fn.read_dir.html
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir>;

    /// Removes the file at `path`.
    /// This is based on [`std::fs::remove_file`].
    ///
    /// [`std::fs::remove_file`]: https://doc.rust-lang.org/std/fs/fn.remove_file.html
    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// Copies the file at path `from` to the path `to`.
    /// This is based on [`std::fs::copy`].
    ///
    /// [`std::fs::copy`]: https://doc.rust-lang.org/std/fs/fn.copy.html
    fn copy_file<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>;

    /// Renames a file or directory.
    /// If both `from` and `to` are files, `to` will be replaced.
    /// Based on [`std::fs::rename`].
    ///
    /// [`std::fs::rename`]: https://doc.rust-lang.org/std/fs/fn.rename.html
    fn rename<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>;

    /// Returns the canonical, absolute form of a path with all intermediate components
    /// normalized and symbolic links resolved.
    /// This is based on [`fs::canonicalize`].
    ///
    /// [`fs::canonicalize`]: https://doc.rust-lang.org/std/fs/fn.canonicalize.html
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;
}

/// Entries returned by the ReadDir iterator.
/// This is based on [`fs::DirEntry`].
///
/// [`fs::DirEntry`]: https://doc.rust-lang.org/std/fs/struct.DirEntry.html
pub trait DirEntry {
    /// Returns the bare file name of this directory entry without any other leading path component.
    /// This is based on [`fs::DirEntry::file_name`].
    ///
    /// [`fs::DirEntry::file_name`]: https://doc.rust-lang.org/std/fs/struct.DirEntry.html#method.file_name
    fn file_name(&self) -> OsString;

    /// Returns the full path to the file that this entry represents.
    /// This is based on [`fs::DirEntry::path`].
    ///
    /// [`fs::DirEntry::path`]: https://doc.rust-lang.org/std/fs/struct.DirEntry.html#method.path
    fn path(&self) -> PathBuf;
}

pub trait ReadDir<T: DirEntry>: Iterator<Item = Result<T>> {}

/// Provides functions which are not modelled as traits in [`fs::File`]
///
/// [`fs::File`]: https://doc.rust-lang.org/std/fs/struct.File.html
pub trait FileExt {
    type Metadata: Metadata;

    /// Queries metadata about the underlying file.
    /// This is based on [`fs::File::metadata`].
    ///
    /// [`fs::File::metadata`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.metadata
    fn metadata(&self) -> Result<Self::Metadata>;

    /// Truncates or extends the underlying file, updating the size of this file to become size.
    /// This is based on [`fs::File::set_len`]
    ///
    /// [`fs::File::set_len`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.set_len
    fn set_len(&self, size: u64) -> Result<()>;

    /// Attempts to sync all OS-internal metadata to disk.
    /// This is based on [`fs::File::sync_all`]
    ///
    /// [`fs::File::sync_all`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all
    fn sync_all(&self) -> Result<()>;

    /// This function is similar to sync_all, except that it may not synchronize file metadata
    /// to the filesystem.
    /// This is based on [`fs::File::sync_data`]
    ///
    /// [`fs::File::sync_data`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_data
    fn sync_data(&self) -> Result<()>;
}

/// Metadata information about a file.
/// This is based on [`fs::Metadata`].
///
/// [`fs::Metadata`]: https://doc.rust-lang.org/std/fs/struct.Metadata.html
#[allow(clippy::len_without_is_empty)]
pub trait Metadata: fmt::Debug {
    type Permissions: Permissions;

    /// Returns true if this metadata is for a directory.
    /// This is based on [`fs::Metadata::is_dir`].
    ///
    /// [`fs::Metadata::is_dir`]: https://doc.rust-lang.org/std/fs/struct.Metadata.html#method.is_dir
    fn is_dir(&self) -> bool;

    /// Returns true if this metadata is for a regular file.
    /// This is based on [`fs::Metadata::is_file`].
    ///
    /// [`fs::Metadata::is_file`]: https://doc.rust-lang.org/std/fs/struct.Metadata.html#method.is_file
    fn is_file(&self) -> bool;

    /// Returns the size of the file, in bytes, this metadata is for.
    /// This is based on [`fs::Metadata::len`].
    ///
    /// [`fs::Metadata::len`]: https://doc.rust-lang.org/std/fs/struct.Metadata.html#method.len
    fn len(&self) -> u64;

    /// Returns the permissions of the file this metadata is for.
    /// This is based on [`fs::Metadata::permissions`].
    ///
    /// [`fs::Metadata::permissions`]: https://doc.rust-lang.org/std/fs/struct.Metadata.html?search=#method.permissions
    fn permissions(&self) -> Self::Permissions;
}

/// Representation of the various permissions on a file.
/// This is based on [`fs::Permissions`].
///
/// [`fs::Permissions`]: https://doc.rust-lang.org/std/fs/struct.Permissions.html
pub trait Permissions {
    /// Returns true if these permissions describe a readonly (unwritable) file.
    /// This is based on [`fs::Permissions::readonly`].
    ///
    /// [`fs::Permissions::readonly`]: https://doc.rust-lang.org/std/fs/struct.Permissions.html#method.readonly
    fn readonly(&self) -> bool;

    /// Modifies the readonly flag for this set of permissions.
    /// This is based on [`fs::Permissions::set_readonly`].
    ///
    /// [`fs::Permissions::set_readonly`]: https://doc.rust-lang.org/std/fs/struct.Permissions.html#method.set_readonly
    fn set_readonly(&mut self, readonly: bool);

    /// Returns the underlying raw st_mode bits that contain the standard Unix permissions for this file.
    /// This is based on [`os::unix::fs::PermissionsExt::mode`].
    ///
    /// [`os::unix::fs::PermissionsExt::mode`]: https://doc.rust-lang.org/std/os/unix/fs/trait.PermissionsExt.html#tymethod.mode
    #[cfg(unix)]
    fn mode(&self) -> u32;

    /// Sets the underlying raw bits for this set of permissions.
    /// This is based on [`os::unix::fs::PermissionsExt::set_mode`].
    ///
    /// [`os::unix::fs::PermissionsExt::set_mode`]: https://doc.rust-lang.org/std/os/unix/fs/trait.PermissionsExt.html#tymethod.set_mode
    #[cfg(unix)]
    fn set_mode(&mut self, mode: u32);

    /// Creates a new instance of Permissions from the given set of Unix permission bits.
    /// This is based on [`os::unix::fs::PermissionsExt::from_mode`].
    ///
    /// [`os::unix::fs::PermissionsExt::from_mode`]: https://doc.rust-lang.org/std/os/unix/fs/trait.PermissionsExt.html#tymethod.from_mode
    #[cfg(unix)]
    fn from_mode(mode: u32) -> Self;
}

#[cfg(feature = "temp")]
/// Tracks a temporary directory that will be deleted once the struct goes out of scope.
pub trait TempDir {
    /// Returns the [`Path`] of the temporary directory.
    ///
    /// [`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
    fn path(&self) -> &Path;
}

#[cfg(feature = "temp")]
pub trait TempFileSystem: Clone + Send + Sync {
    type TempDir: TempDir;

    /// Creates a new temporary directory.
    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir>;
}

/// Options and flags which can be used to configure how a file is opened.
/// This is based on [`fs::OpenOptions`].
///
/// [`fs::OpenOptions`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpenOptions {
    append: bool,
    create: bool,
    create_new: bool,
    read: bool,
    truncate: bool,
    write: bool,
}

impl OpenOptions {
    /// Constructs an OpenOptions with all options set to false.
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the option for the append mode.
    /// This is based on [`fs::OpenOptions::append`].
    ///
    /// [`fs::OpenOptions::append`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.append
    pub fn append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }

    /// Sets the option for creating a new file.
    /// This is based on [`fs::OpenOptions::create`].
    ///
    /// [`fs::OpenOptions::create`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create
    pub fn create(mut self, create: bool) -> Self {
        self.create = create;
        self
    }

    /// Sets the option to always create a new file.
    /// This is based on [`fs::OpenOptions::create_new`].
    ///
    /// [`fs::OpenOptions::create_new`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new
    pub fn create_new(mut self, create_new: bool) -> Self {
        self.create_new = create_new;
        self
    }

    /// Sets the option for read access.
    /// This is based on [`fs::OpenOptions::read`].
    ///
    /// [`fs::OpenOptions::read`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.read
    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }

    /// Sets the option for truncating a previous file.
    /// This is based on [`fs::OpenOptions::truncate`].
    ///
    /// [`fs::OpenOptions::truncate`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.truncate
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option for write access.
    /// This is based on [`fs::OpenOptions::write`].
    ///
    /// [`fs::OpenOptions::write`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.write
    pub fn write(mut self, write: bool) -> Self {
        self.write = write;
        self
    }
}
