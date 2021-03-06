use std::ffi::{OsStr, OsString};
use std::io::{self, Result, SeekFrom};
use std::iter::Iterator;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::{Arc, Mutex, MutexGuard};
use std::vec::IntoIter;
use std::cmp::min;
use std::io::ErrorKind;
use std::borrow::Cow;
use node::{SharedMode};
use registry::create_error;
use crate::OpenOptions;

use super::{FileSystem, FileExt, Metadata, Permissions};
#[cfg(feature = "temp")]
use super::{TempDir, TempFileSystem};

#[cfg(feature = "temp")]
pub use self::tempdir::FakeTempDir;

use self::registry::Registry;

mod node;
mod registry;
#[cfg(feature = "temp")]
mod tempdir;

/// An in-memory file system.
#[derive(Clone, Debug, Default)]
pub struct FakeFileSystem {
    registry: Arc<Mutex<Registry>>,
}

fn to_absolute_path<F>(mut path: Cow<'_, Path>, get_current_dir: F) -> Cow<'_, Path>
where F: FnOnce() -> Result<PathBuf> {
    if path.is_relative() {
        path = get_current_dir()
            .unwrap_or_else(|_| PathBuf::from(MAIN_SEPARATOR.to_string()))
            .join(path)
            .into();
    }
    path
}

impl FakeFileSystem {
    pub fn new() -> Self {
        let registry = Registry::new();

        FakeFileSystem {
            registry: Arc::new(Mutex::new(registry)),
        }
    }

    fn apply<F, T>(&self, path: &Path, f: F) -> T
    where
        F: FnOnce(&MutexGuard<Registry>, &Path) -> T,
    {
        let registry = self.registry.lock().unwrap();
        let path = to_absolute_path(Cow::from(path), || registry.current_dir());

        f(&registry, &path)
    }

    fn apply_mut<F, T>(&self, path: &Path, mut f: F) -> T
    where
        F: FnMut(&mut MutexGuard<Registry>, &Path) -> T,
    {
        let mut registry = self.registry.lock().unwrap();
        let path = to_absolute_path(Cow::from(path), || registry.current_dir());

        f(&mut registry, &path)
    }

    fn apply_mut_from_to<F, T>(&self, from: &Path, to: &Path, mut f: F) -> T
    where
        F: FnMut(&mut MutexGuard<Registry>, &Path, &Path) -> T,
    {
        let mut registry = self.registry.lock().unwrap();
        let from = to_absolute_path(Cow::from(from), || registry.current_dir());
        let to   = to_absolute_path(Cow::from(to  ), || registry.current_dir());

        f(&mut registry, &from, &to)
    }

    // Opens an existing file as write-only.
    // Does not modify the file on open.
    fn open_writable<P: AsRef<Path>>(&self, path: P) -> Result<FakeOpenFile> {
        self.apply(path.as_ref(), |r, p| {
            r.get_file_if_writable(p)
                .map(|f| FakeOpenFile::new(f, AccessMode::Write))
        })
    }

    // Creates a new file as write-only.
    // Fails if the file already exists.
    fn create_new<P: AsRef<Path>>(&self, path: P) -> Result<FakeOpenFile> {
        self.apply_mut(path.as_ref(), |r, p| {
            // make sure file does not exist
            // careful, check presence in a way that works even if
            // we have no access to the file.
            if r.readonly(p).is_ok() {
                return Err(io::Error::new(ErrorKind::AlreadyExists, "Already Exists"));
            }
            // create it
            r.write_file(p, &[])?;
            r.get_file_if_writable(p)
                .map(|f| FakeOpenFile::new(f, AccessMode::Write))
        })
    }

    // Opens an existing file as write-only.
    // Truncates on open.
    // Fails if the file does not exist.
    fn overwrite<P: AsRef<Path>>(&self, path: P) -> Result<FakeOpenFile> {
        self.apply(path.as_ref(), |r, p| {
            // overwite file
            // this ensure the file exists and we have
            // write access.
            r.overwrite_file(p, &[])?;
            let f = r.get_file_if_writable(p)?;
            Ok(FakeOpenFile::new(f, AccessMode::Write))
        })
    }
}

impl FileSystem for FakeFileSystem {
    type DirEntry = DirEntry;
    type ReadDir = ReadDir;
    type File = FakeOpenFile;
    type Permissions = FakePermissions;
    type Metadata = FakeMetadata;

    fn open<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        self.apply(path.as_ref(), |r, p|
            r.get_file_if_readable(p)
                .map(|f| FakeOpenFile::new(f, AccessMode::Read)))
    }

    fn create<P: AsRef<Path>>(&self, path: P) -> Result<Self::File> {
        self.apply_mut(path.as_ref(), |r, p| {
            r.write_file(p, &[])?;
            let f = r.get_file_if_writable(p)?;
            Ok(FakeOpenFile::new(f, AccessMode::Write))
        })
    }

    fn open_with_options<P: AsRef<Path>>(&self, path: P, o: &OpenOptions) -> Result<Self::File> {

        let o_create = OpenOptions::new().create(true).truncate(true).write(true);
        let o_open = OpenOptions::new().read(true);
        let o_open_writable = OpenOptions::new().write(true);
        let o_create_new = OpenOptions::new().create_new(true).write(true);
        let o_overwrite = OpenOptions::new().truncate(true).write(true);

        match o {
            o if *o == o_create         => self.create(path),
            o if *o == o_open           => self.open(path),
            o if *o == o_open_writable  => self.open_writable(path),
            o if *o == o_create_new     => self.create_new(path),
            o if *o == o_overwrite      => self.overwrite(path),
             _ => Err(io::Error::new(ErrorKind::InvalidInput,
                        format!("FakeFileSystem: Unsupported {:?}", o))),
        }
    }

    #[cfg(unix)]
    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()>
    {
        self.apply(path.as_ref(), |r, p| r.set_mode(p, perm.mode()))
    }

    #[cfg(not(unix))]
    fn set_permissions<P: AsRef<Path>>(&self, path: P, perm: Self::Permissions) -> Result<()>
    {
        self.apply(path.as_ref(), |r, p| r.set_readonly(p, perm.readonly()))
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Self::Metadata> {
        self.apply(path.as_ref(), |r, p|
            if r.is_file(p) {
                r.get_file(p).map(FakeMetadata::from)
            } else {
                r.get_dir(p).map(FakeMetadata::from)
            }
        )
    }

    fn current_dir(&self) -> Result<PathBuf> {
        let registry = self.registry.lock().unwrap();
        registry.current_dir()
    }

    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.set_current_dir(p.to_path_buf()))
    }

    fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        self.apply(path.as_ref(), |r, p| r.is_dir(p))
    }

    fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.apply(path.as_ref(), |r, p| r.is_file(p))
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.create_dir(p))
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.create_dir_all(p))
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.remove_dir(p))
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.remove_dir_all(p))
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadDir> {
        let path = path.as_ref();

        self.apply(path, |r, p| r.read_dir(p)).map(|entries| {
            let entries = entries
                .iter()
                .map(|e| {
                    let file_name = e.file_name().unwrap_or_else(|| e.as_os_str());

                    Ok(DirEntry::new(path, &file_name))
                })
                .collect();

            ReadDir::new(entries)
        })
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.apply_mut(path.as_ref(), |r, p| r.remove_file(p))
    }

    fn copy_file<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        self.apply_mut_from_to(from.as_ref(), to.as_ref(), |r, from, to| {
            r.copy_file(from, to)
        })
    }

    fn rename<P, Q>(&self, from: P, to: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        self.apply_mut_from_to(from.as_ref(), to.as_ref(), |r, from, to| r.rename(from, to))
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        // special case: empty paths must always fail
        if path.as_os_str().is_empty() {
            return Err(create_error(ErrorKind::NotFound));
        }
        self.apply(path, |r, p| r.canonicalize_path(p))
    }
}

/// How a `fs::File` is accessed.
///
#[derive(Debug, PartialEq)]
enum AccessMode {
    Read,
    Write,
}

#[derive(Debug)]
pub struct FakeOpenFile {
    /// Pointer to the file we have open
    f: node::File,
    pos: usize,
    access_mode: AccessMode,
}

impl FakeOpenFile {
    fn new(file: &node::File, access_mode: AccessMode) -> Self {
        FakeOpenFile {
            f: file.clone(),
            pos: 0,
            access_mode,
        }
    }
    fn verify_access(&self, access_mode: AccessMode) -> Result<()> {
        if access_mode != self.access_mode {
            Err(create_error(ErrorKind::Other))
        } else {
            Ok(())
        }
    }
}

impl io::Read for FakeOpenFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.verify_access(AccessMode::Read)?;
        let contents = self.f.contents.borrow();
        let pos = self.pos;
        // If the underlying file has shrunk, the offset could
        // point to beyond eof.
        let len = if pos < contents.len() {
            min(contents.len() - pos, buf.len())
        } else {
            0
        };
        if len > 0 {
            buf[..len].copy_from_slice(&contents[pos..pos+len]);
            self.pos += len;
        }
        Ok(len)
    }
}

impl io::Seek for FakeOpenFile {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let pos = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(offs) => self.f.contents.borrow().len() as i64 + offs,
            SeekFrom::Current(offs) => self.pos as i64 + offs,
        };
        if pos >= 0 {
            self.pos = pos as usize;
            Ok(pos as u64)
        } else {
            // it's an error to seek before byte 0
            Err(create_error(ErrorKind::InvalidInput))
        }
    }
}

impl io::Write for FakeOpenFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.verify_access(AccessMode::Write)?;
        let mut contents = self.f.contents.borrow_mut();
        let pos = self.pos;
        // if pos points beyond eof, resize contents to pos and pad with zeros
        if pos > contents.len() {
            contents.resize(pos, 0);
        }
        let copy_len = min(buf.len(), contents.len() - pos);
        contents[pos..pos+copy_len].copy_from_slice(&buf[..copy_len]);
        contents.extend_from_slice(&buf[copy_len..]);
        self.pos += buf.len();
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl FileExt for FakeOpenFile {
    type Metadata = FakeMetadata;

    fn metadata(&self) -> Result<Self::Metadata> {
        Ok(FakeMetadata::from(&self.f))
    }
    fn set_len(&self, size: u64) -> Result<()> {
        self.verify_access(AccessMode::Write)?;
        let mut contents = self.f.contents.borrow_mut();
        contents.resize(size as usize, 0);
        Ok(())
    }
    fn sync_all(&self) -> Result<()> {
        Ok(())
    }
    fn sync_data(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct FakeMetadata {
    len: u64,
    permissions: FakePermissions,
    is_dir: bool,
}

impl From<&node::File> for FakeMetadata {
    fn from(f: &node::File) -> Self {
        FakeMetadata {
            len: f.contents.borrow().len() as u64,
            permissions: FakePermissions::from(&f.mode),
            is_dir: false,
        }
    }
}

impl From<&node::Dir> for FakeMetadata {
    fn from(d: &node::Dir) -> Self {
        FakeMetadata {
            len: 4096,
            permissions: FakePermissions::from(&d.mode),
            is_dir: true,
        }
    }
}

impl Metadata for FakeMetadata {
    type Permissions = FakePermissions;

    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn is_file(&self) -> bool {
        !self.is_dir
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn permissions(&self) -> Self::Permissions {
        self.permissions.clone()
    }
}

#[derive(Debug, Clone)]
pub struct FakePermissions(u32);

impl From<&SharedMode> for FakePermissions {
    fn from(mode: &SharedMode) -> FakePermissions {
        FakePermissions(mode.get())
    }
}

impl Permissions for FakePermissions {
    fn readonly(&self) -> bool {
        (self.0 & 0o222) == 0
    }
    fn set_readonly(&mut self, readonly: bool) {
        if readonly {
            self.0 &= !0o222;
        } else {
            self.0 |= 0o222;
        }
    }
    #[cfg(unix)]
    fn mode(&self) -> u32 {
        self.0
    }
    #[cfg(unix)]
    fn set_mode(&mut self, mode: u32) {
        self.0 = mode;
    }
    #[cfg(unix)]
    fn from_mode(mode: u32) -> Self {
        FakePermissions(mode)
    }
}

#[derive(Debug)]
pub struct DirEntry {
    parent: PathBuf,
    file_name: OsString,
}

impl DirEntry {
    fn new<P, S>(parent: P, file_name: S) -> Self
    where
        P: AsRef<Path>,
        S: AsRef<OsStr>,
    {
        DirEntry {
            parent: parent.as_ref().to_path_buf(),
            file_name: file_name.as_ref().to_os_string(),
        }
    }
}

impl crate::DirEntry for DirEntry {
    fn file_name(&self) -> OsString {
        self.file_name.clone()
    }

    fn path(&self) -> PathBuf {
        self.parent.join(&self.file_name)
    }
}

#[derive(Debug)]
pub struct ReadDir(IntoIter<Result<DirEntry>>);

impl ReadDir {
    fn new(entries: Vec<Result<DirEntry>>) -> Self {
        ReadDir(entries.into_iter())
    }
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl crate::ReadDir<DirEntry> for ReadDir {}

#[cfg(feature = "temp")]
impl TempFileSystem for FakeFileSystem {
    type TempDir = FakeTempDir;

    fn temp_dir<S: AsRef<str>>(&self, prefix: S) -> Result<Self::TempDir> {
        let base = std::env::temp_dir();
        let dir = FakeTempDir::new(Arc::downgrade(&self.registry), &base, prefix.as_ref());

        self.create_dir_all(&dir.path()).and(Ok(dir))
    }
}
