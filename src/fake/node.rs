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

#[derive(Debug)]
pub struct File {
    pub contents: SharedContents,
    pub mode: u32,
}

impl File {
    pub fn new(contents: Vec<u8>) -> Self {
        File {
            contents: SharedContents::new(contents),
            mode: 0o644,
        }
    }
}

#[derive(Debug, Default)]
pub struct Dir {
    pub mode: u32,
}

impl Dir {
    pub fn new() -> Self {
        Dir { mode: 0o644 }
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
