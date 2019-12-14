use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

/// A reference-counted pointer to the contents of a file.
///
/// `clone` just creates another pointer, it does not Clone
/// the contents itself.
///
#[derive(Debug, Clone)]
pub struct SharedContents(Arc<Mutex<Vec<u8>>>);

impl SharedContents {
    fn new(contents: Vec<u8>) -> Self {
        SharedContents(Arc::new(Mutex::new(contents)))
    }
    /// Immutably borrow the file contents pointed to.
    pub fn borrow(&self) -> impl Deref<Target=Vec<u8>> + '_ {
        self.0.lock().unwrap()
    }
    /// Mutably borrow the file contents pointed to.
    pub fn borrow_mut(&self) -> impl DerefMut<Target=Vec<u8>> + '_ {
        self.0.lock().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct SharedMode(Arc<Mutex<u32>>);

impl SharedMode {
    fn new(mode: u32) -> Self {
        SharedMode(Arc::new(Mutex::new(mode)))
    }

    pub fn get(&self) -> u32 {
        *self.0.lock().unwrap()
    }
    pub fn set(&self, mode: u32) {
        *self.0.lock().unwrap() = mode;
    }
    pub fn can_read(&self) -> bool {
        (*self.0.lock().unwrap() & 0o444) != 0
    }
    pub fn can_write(&self) -> bool {
        (*self.0.lock().unwrap() & 0o222) != 0
    }
    pub fn make_readonly(&self, readonly: bool) {
        let mut mode = self.0.lock().unwrap();
        if readonly {
            *mode &= !0o222;
        } else {
            *mode |= 0o222;
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub contents: SharedContents,
    pub mode: SharedMode,
}

impl File {
    pub fn new(contents: Vec<u8>) -> Self {
        File {
            contents: SharedContents::new(contents),
            mode: SharedMode::new(0o644),
        }
    }
}

#[derive(Debug)]
pub struct Dir {
    pub mode: SharedMode,
}

impl Dir {
    pub fn new() -> Self {
        Dir { mode: SharedMode::new(0o644) }
    }
}

#[derive(Debug)]
pub enum Node {
    File(File),
    Dir(Dir),
}

impl Node {
    pub fn is_file(&self) -> bool {
        match *self {
            Self::File(_) => true,
            _ => false,
        }
    }

    pub fn is_dir(&self) -> bool {
        match *self {
            Self::Dir(_) => true,
            _ => false,
        }
    }
}
