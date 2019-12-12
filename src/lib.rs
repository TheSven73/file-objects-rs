#[cfg(feature = "temp")]
extern crate rand;
#[cfg(feature = "temp")]
extern crate tempdir;

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
    type File: io::Read + io::Seek + io::Write + FileExt + fmt::Debug;

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

    /// Determines whether the path exists and points to a directory.
    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool;
    /// Determines whether the path exists and points to a file.
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

    /// Writes `buf` to a new file at `path`.
    ///
    /// # Errors
    ///
    /// * A file or directory already exists at `path`.
    /// * The parent directory of `path` does not exist.
    /// * Current user has insufficient permissions.
    fn create_file<P, B>(&self, path: P, buf: B) -> Result<()>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>;
    /// Writes `buf` to a new or existing file at `buf`.
    /// This will overwrite any contents that already exist.
    ///
    /// # Errors
    ///
    /// * The parent directory of `path` does not exist.
    /// * Current user has insufficient permissions.
    fn write_file<P, B>(&self, path: P, buf: B) -> Result<()>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>;
    /// Writes `buf` to an existing file at `buf`.
    /// This will overwrite any contents that already exist.
    ///
    /// # Errors
    ///
    /// * No file `file` does not exist.
    /// * The node at `file` is a directory.
    /// * Current user has insufficient permissions.
    fn overwrite_file<P, B>(&self, path: P, buf: B) -> Result<()>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>;
    /// Returns the contents of `path`.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * `path` is a directory.
    /// * Current user has insufficient permissions.
    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>>;
    /// Returns the contents of `path` as a string.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * `path` is a directory.
    /// * Current user has insufficient permissions.
    /// * Contents are not valid UTF-8
    fn read_file_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String>;
    /// Writes the contents of `path` into the buffer. If successful, returns
    /// the number of bytes that were read.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * `path` is a directory.
    /// * Current user has insufficient permissions.
    fn read_file_into<P, B>(&self, path: P, buf: B) -> Result<usize>
    where
        P: AsRef<Path>,
        B: AsMut<Vec<u8>>;
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

    /// Returns `true` if `path` is a readonly file.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * Current user has insufficient permissions.
    fn readonly<P: AsRef<Path>>(&self, path: P) -> Result<bool>;
    /// Sets or unsets the readonly flag of `path`.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * Current user has insufficient permissions.
    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result<()>;

    /// Returns the length of the node at the path
    /// or 0 if the node does not exist.
    fn len<P: AsRef<Path>>(&self, path: P) -> u64;

    /// Returns the canonical, absolute form of a path with all intermediate components
    /// normalized and symbolic links resolved.
    /// This is based on [`fs::canonicalize`].
    ///
    /// [`fs::canonicalize`]: https://doc.rust-lang.org/std/fs/fn.canonicalize.html
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;
}

pub trait DirEntry {
    fn file_name(&self) -> OsString;
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
pub trait Metadata {
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
}

#[cfg(unix)]
pub trait UnixFileSystem {
    /// Returns the current mode bits of `path`.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * Current user has insufficient permissions.
    fn mode<P: AsRef<Path>>(&self, path: P) -> Result<u32>;
    /// Sets the mode bits of `path`.
    ///
    /// # Errors
    ///
    /// * `path` does not exist.
    /// * Current user has insufficient permissions.
    fn set_mode<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result<()>;
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
#[derive(Clone, Debug, Default)]
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
