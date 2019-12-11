#[derive(Debug)]
pub struct File {
    pub contents: Vec<u8>,
    pub mode: u32,
}

impl File {
    pub fn new(contents: Vec<u8>) -> Self {
        File {
            contents,
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
