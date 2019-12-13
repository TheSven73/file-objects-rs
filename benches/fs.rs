#[macro_use]
extern crate bencher;
extern crate filesystem;

use std::io::{Write, SeekFrom, Seek, Read};
use std::path::PathBuf;

use bencher::Bencher;
use filesystem::{FileSystem, FakeFileSystem};

fn create_file_absolute(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = fs.current_dir().unwrap().join("hello.txt");
    bench.iter( || {
        fs.create_file(&path, b"").unwrap();
        fs.remove_file(&path).unwrap();
    });
}

fn create_file_relative(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = PathBuf::from("hello.txt");
    bench.iter( || {
        fs.create_file(&path, b"").unwrap();
        fs.remove_file(&path).unwrap();
    });
}

fn create_file_deep_relative_path(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let deep: PathBuf = std::iter::repeat("test").take(20).collect();
    fs.create_dir_all(&deep).unwrap();
    let path = deep.join("test.txt");
    bench.iter( || {
        fs.create_file(&path, b"").unwrap();
        fs.remove_file(&path).unwrap();
    });
}

fn create_file_deep_absolute_path(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let deep: PathBuf = std::iter::repeat("test").take(20).collect();
    let deep = fs.current_dir().unwrap().join(deep);
    fs.create_dir_all(&deep).unwrap();
    let path = deep.join("test.txt");
    bench.iter( || {
        fs.create_file(&path, b"").unwrap();
        fs.remove_file(&path).unwrap();
    });
}

fn create_file_long_filename(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let file_name = ["test"].iter().cloned().take(20).collect::<Vec<_>>().join("");
    let path = PathBuf::from(file_name);
    bench.iter( || {
        fs.create_file(&path, "").unwrap();
        fs.remove_file(&path).unwrap();
    });
}

fn write_file(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = fs.current_dir().unwrap().join("hello.txt");
    let contents = b"hello world";
    bench.iter( || {
        fs.write_file(&path, contents).unwrap()
    });
}

fn read_file(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = fs.current_dir().unwrap().join("hello.txt");
    fs.create_file(&path, b"hello world").unwrap();
    bench.iter( || {
        let mut buf = vec![];
        fs.open(&path).unwrap().read_to_end(&mut buf).unwrap();
    });
}

fn seek_in_reader(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = fs.current_dir().unwrap().join("hello.txt");
    fs.create_file(&path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();
    bench.iter( || {
        reader.seek(SeekFrom::Start(0)).unwrap();
    });
}

fn create_dir_relative(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = PathBuf::from("test");
    bench.iter( || {
        fs.create_dir(&path).unwrap();
        fs.remove_dir(&path).unwrap();
    });
}

fn create_dir_absolute(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let path = PathBuf::from("test");
    let path = fs.current_dir().unwrap().join(&path);
    bench.iter( || {
        fs.create_dir(&path).unwrap();
        fs.remove_dir(&path).unwrap();
    });
}

fn open_file_with_large_fs(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    // put a thousand small files in
    let root = fs.current_dir().unwrap();
    let contents = b"hello world";
    for id in 0..1000 {
        let path = root.join(id.to_string());
        //fs.create(&path).unwrap().write_all(contents).unwrap();
        fs.write_file(&path, contents).unwrap();
    }
    bench.iter( || {
        let path = root.join("65");
        fs.open(&path).unwrap()
    });
}

fn read_dir(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let root = fs.current_dir().unwrap();
    for id in 0..100 {
        let path = root.join(id.to_string());
        fs.create_dir(&path).unwrap();
    }
    bench.iter( || {
        let dir = fs.read_dir(&root).unwrap();
        dir.count()
    });
}

fn is_dir(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let root = fs.current_dir().unwrap();
    let dir = root.join("test");
    fs.create_dir(&dir).unwrap();
    bench.iter( || {
        fs.is_dir(&dir)
    });
}

fn copy_file(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let root = fs.current_dir().unwrap();
    let path1 = root.join("test1.txt");
    let path2 = root.join("test2.txt");
    //fs.create(&path1).unwrap().write_all(b"the quick brown fox").unwrap();
    fs.create_file(&path1, b"the quick brown fox").unwrap();
    bench.iter( || {
        fs.copy_file(&path1, &path2).unwrap()
    });
}

fn rename_file(bench: &mut Bencher) {
    let fs = FakeFileSystem::new();
    let root = fs.current_dir().unwrap();
    let path1 = root.join("test1.txt");
    let path2 = root.join("test2.txt");
    //fs.create(&path1).unwrap().write_all(b"the quick brown fox").unwrap();
    fs.create_file(&path1, b"the quick brown fox").unwrap();
    bench.iter( || {
        fs.rename(&path1, &path2).unwrap();
        fs.rename(&path2, &path1).unwrap();
    });
}

benchmark_group!(benches,
    create_file_absolute,
    create_file_relative,
    create_file_deep_relative_path,
    create_file_deep_absolute_path,
    create_file_long_filename,
    write_file,
    read_file,
    seek_in_reader,
    create_dir_relative,
    create_dir_absolute,
    open_file_with_large_fs,
    read_dir,
    is_dir,
    copy_file,
    rename_file,
);
benchmark_main!(benches);
