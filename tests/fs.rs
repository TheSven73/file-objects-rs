use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use file_objects_rs::{DirEntry, FakeFileSystem, FileSystem, OsFileSystem, TempDir, TempFileSystem};
use file_objects_rs::{FileExt, Metadata, OpenOptions, Permissions};

macro_rules! make_test {
    ($test:ident, $fs:expr) => {
        #[test]
        fn $test() {
            let fs = $fs();
            let temp_dir = fs.temp_dir("test").unwrap();

            super::$test(&fs, temp_dir.path());
        }
    };
}

macro_rules! test_fs {
    ($name:ident, $fs:expr) => {
        mod $name {
            use super::*;

            make_test!(set_current_dir_fails_if_node_does_not_exists, $fs);
            make_test!(set_current_dir_fails_if_node_is_a_file, $fs);

            make_test!(is_dir_returns_true_if_node_is_dir, $fs);
            make_test!(is_dir_returns_false_if_node_is_file, $fs);
            make_test!(is_dir_returns_false_if_node_does_not_exist, $fs);

            make_test!(is_file_returns_true_if_node_is_file, $fs);
            make_test!(is_file_returns_false_if_node_is_dir, $fs);
            make_test!(is_file_returns_false_if_node_does_not_exist, $fs);

            make_test!(create_dir_creates_new_dir, $fs);
            make_test!(create_dir_fails_if_dir_already_exists, $fs);
            make_test!(create_dir_fails_if_parent_does_not_exist, $fs);

            make_test!(create_dir_all_creates_dirs_in_path, $fs);
            make_test!(create_dir_all_still_succeeds_if_any_dir_already_exists, $fs);

            make_test!(remove_dir_deletes_dir, $fs);
            make_test!(remove_dir_does_not_affect_parent, $fs);
            make_test!(remove_dir_fails_if_node_does_not_exist, $fs);
            make_test!(remove_dir_fails_if_node_is_a_file, $fs);
            make_test!(remove_dir_fails_if_dir_is_not_empty, $fs);

            make_test!(remove_dir_all_removes_dir_and_contents, $fs);
            make_test!(remove_dir_all_fails_if_node_is_a_file, $fs);
            #[cfg(unix)]
            make_test!(
                remove_dir_all_removes_dir_and_contents_if_descendant_not_writable,
                $fs
            );
            #[cfg(unix)]
            make_test!(
                remove_dir_all_removes_dir_and_contents_if_descendant_not_executable,
                $fs
            );
            #[cfg(unix)]
            make_test!(remove_dir_all_fails_if_descendant_not_readable, $fs);

            make_test!(read_dir_returns_dir_entries, $fs);
            make_test!(read_dir_fails_if_node_does_not_exist, $fs);
            make_test!(read_dir_fails_if_node_is_a_file, $fs);

            make_test!(write_file_writes_to_new_file, $fs);
            make_test!(write_file_overwrites_contents_of_existing_file, $fs);
            make_test!(write_file_fails_if_file_is_readonly, $fs);
            make_test!(write_file_fails_if_node_is_a_directory, $fs);

            make_test!(overwrite_file_overwrites_contents_of_existing_file, $fs);
            make_test!(overwrite_file_fails_if_node_does_not_exist, $fs);
            make_test!(overwrite_file_fails_if_file_is_readonly, $fs);
            make_test!(overwrite_file_fails_if_node_is_a_directory, $fs);

            make_test!(read_file_returns_contents_as_bytes, $fs);
            make_test!(read_file_fails_if_file_does_not_exist, $fs);

            make_test!(read_file_to_string_returns_contents_as_string, $fs);
            make_test!(read_file_to_string_fails_if_file_does_not_exist, $fs);
            make_test!(read_file_to_string_fails_if_contents_are_not_utf8, $fs);

            make_test!(read_file_into_writes_bytes_to_buffer, $fs);
            make_test!(read_file_into_fails_if_file_does_not_exist, $fs);

            make_test!(open_object_writes_bytes_to_buffer, $fs);
            make_test!(open_object_fails_if_file_does_not_exist, $fs);

            make_test!(create_file_writes_to_new_file, $fs);
            make_test!(create_file_fails_if_file_already_exists, $fs);

            make_test!(remove_file_removes_a_file, $fs);
            make_test!(remove_file_fails_if_file_does_not_exist, $fs);
            make_test!(remove_file_fails_if_node_is_a_directory, $fs);

            make_test!(copy_file_copies_a_file, $fs);
            make_test!(copy_file_overwrites_destination_file, $fs);
            make_test!(copy_file_fails_if_original_file_does_not_exist, $fs);
            make_test!(copy_file_fails_if_destination_file_is_readonly, $fs);
            make_test!(copy_file_fails_if_original_node_is_directory, $fs);
            make_test!(copy_file_fails_if_destination_node_is_directory, $fs);

            make_test!(rename_renames_a_file, $fs);
            make_test!(rename_renames_a_directory, $fs);
            make_test!(rename_overwrites_destination_file, $fs);
            make_test!(rename_overwrites_empty_destination_directory, $fs);
            make_test!(rename_renames_all_descendants, $fs);
            make_test!(rename_fails_if_original_path_does_not_exist, $fs);
            make_test!(
                rename_fails_if_original_and_destination_are_different_types,
                $fs
            );
            make_test!(rename_fails_if_destination_directory_is_not_empty, $fs);

            make_test!(readonly_returns_write_permission, $fs);
            make_test!(readonly_fails_if_node_does_not_exist, $fs);

            make_test!(set_readonly_toggles_write_permission_of_file, $fs);
            make_test!(set_readonly_toggles_write_permission_of_dir, $fs);
            make_test!(set_readonly_fails_if_node_does_not_exist, $fs);

            make_test!(len_returns_size_of_file, $fs);

            make_test!(open_objects_read_independently, $fs);
            make_test!(open_object_cannot_open_dir, $fs);
            make_test!(open_object_read_returns_length, $fs);
            make_test!(open_object_reads_chunked, $fs);
            make_test!(open_object_reads_ok_beyond_eof, $fs);
            make_test!(open_object_reads_ok_after_file_deleted, $fs);
            make_test!(open_object_reads_ok_after_file_overwritten, $fs);
            make_test!(open_object_reads_ok_after_parent_dir_deleted, $fs);
            make_test!(open_object_reads_ok_after_file_renamed, $fs);
            make_test!(open_object_reads_ok_after_parent_dir_renamed, $fs);
            make_test!(open_object_reads_ok_after_parent_dir_moved, $fs);
            make_test!(open_object_reads_ok_after_file_updated, $fs);
            make_test!(open_object_reads_ok_after_file_shrunk, $fs);

            make_test!(open_object_can_seek_from_start_then_read, $fs);
            make_test!(open_object_can_seek_from_current_then_read, $fs);
            make_test!(open_object_can_seek_from_end_then_read, $fs);
            make_test!(open_object_fails_if_seeks_before_byte_0, $fs);
            make_test!(open_object_can_seek_and_read_beyond_eof, $fs);

            make_test!(create_objects_write_independently, $fs);
            make_test!(create_object_cannot_overwrite_dir, $fs);
            make_test!(create_object_writes_chunked, $fs);
            make_test!(create_object_writes_ok_beyond_eof, $fs);
            make_test!(create_object_writes_ok_after_file_deleted, $fs);
            make_test!(create_object_writes_ok_after_file_overwritten, $fs);
            make_test!(create_object_writes_ok_after_parent_dir_deleted, $fs);
            make_test!(create_object_writes_ok_after_file_renamed, $fs);
            make_test!(create_object_writes_ok_after_parent_dir_renamed, $fs);
            make_test!(create_object_writes_ok_after_parent_dir_moved, $fs);
            make_test!(create_object_writes_ok_after_file_updated_short, $fs);
            make_test!(create_object_writes_ok_after_file_updated_long, $fs);
            make_test!(create_object_writes_ok_after_file_shrunk, $fs);

            make_test!(create_object_can_seek_then_overwrite, $fs);
            make_test!(create_object_can_seek_then_overwrite_and_extend, $fs);
            make_test!(create_object_can_seek_then_extend, $fs);

            make_test!(create_object_writes_to_new_file, $fs);
            make_test!(create_object_fails_if_file_is_readonly, $fs);

            make_test!(open_object_cannot_write, $fs);
            make_test!(create_object_cannot_read, $fs);

            make_test!(set_len_on_create_object_truncates_file, $fs);
            make_test!(set_len_on_create_object_extends_file, $fs);
            make_test!(set_len_on_create_object_doesnt_change_cursor, $fs);

            make_test!(open_object_metadata_is_file, $fs);
            make_test!(open_object_metadata_has_correct_len, $fs);
            make_test!(open_object_metadata_len_is_immutable, $fs);
            make_test!(create_object_metadata_is_file, $fs);
            make_test!(create_object_metadata_has_correct_len, $fs);
            make_test!(create_object_metadata_len_is_immutable, $fs);

            make_test!(fs_file_metadata_is_file, $fs);
            make_test!(fs_file_metadata_has_correct_len, $fs);
            make_test!(fs_file_metadata_len_is_immutable, $fs);
            make_test!(fs_file_metadata_fails_if_file_doesn_exist, $fs);

            make_test!(fs_dir_metadata_is_dir, $fs);
            make_test!(fs_dir_metadata_has_correct_len, $fs);

            make_test!(writable_object_does_not_create_file, $fs);
            make_test!(writable_object_sets_cursor_to_beginning, $fs);
            make_test!(writable_object_allows_append, $fs);
            make_test!(writable_object_truncates, $fs);
            make_test!(writable_object_allows_write_short, $fs);
            make_test!(writable_object_allows_write_long, $fs);
            make_test!(writable_object_extends_file, $fs);

            make_test!(canonicalize_ok_if_root, $fs);
            make_test!(canonicalize_fails_if_empty, $fs);
            make_test!(canonicalize_dot_is_current_dir, $fs);
            make_test!(canonicalize_ok_if_relative_path, $fs);
            make_test!(canonicalize_ok_if_path_ends_in_dotdot, $fs);
            make_test!(canonicalize_ok_if_file_exists, $fs);
            make_test!(canonicalize_fails_if_file_doesnt_exist, $fs);
            make_test!(canonicalize_ok_with_dotdot_if_paths_exist, $fs);
            make_test!(canonicalize_fails_with_dotdot_if_path_doesnt_exist, $fs);
            make_test!(canonicalize_cant_go_lower_than_root, $fs);

            #[cfg(not(target_os = "macos"))]
            make_test!(canonicalize_fails_if_subpath_is_file, $fs);

            #[cfg(target_os = "macos")]
            make_test!(canonicalize_ok_if_subpath_is_file, $fs);

            #[cfg(unix)]
            make_test!(mode_returns_permissions, $fs);
            #[cfg(unix)]
            make_test!(mode_fails_if_node_does_not_exist, $fs);

            #[cfg(unix)]
            make_test!(set_mode_sets_permissions, $fs);
            #[cfg(unix)]
            make_test!(set_mode_fails_if_node_does_not_exist, $fs);

            make_test!(temp_dir_creates_tempdir, $fs);
            make_test!(temp_dir_creates_unique_dir, $fs);

        }
    };
}

test_fs!(os, OsFileSystem::new);
test_fs!(fake, FakeFileSystem::new);

// Used to be part of the public API.
// Keep around for the tests.
fn read_file<T: FileSystem, P: AsRef<Path>>(fs: &T, path: P) -> io::Result<Vec<u8>> {
    let mut reader = fs.open(path)?;
    let mut result = vec![];
    reader.read_to_end(&mut result)?;
    Ok(result)
}

// Used to be part of the public API.
// Keep around for the tests.
fn read_file_to_string<T: FileSystem, P: AsRef<Path>>(fs: &T, path: P) -> io::Result<String> {
    let mut reader = fs.open(path)?;
    let mut result = vec![];
    reader.read_to_end(&mut result)?;
    String::from_utf8(result)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid Data"))
}

// Used to be part of the public API.
// Keep around for the tests.
fn read_file_into<T, P, B>(fs: &T, path: P, mut buf: B) -> io::Result<usize>
        where
            T: FileSystem,
            P: AsRef<Path>,
            B: AsMut<Vec<u8>> {

    let mut reader = fs.open(path)?;
    reader.read_to_end(buf.as_mut())
}

// Used to be part of the public API.
// Keep around for the tests.
fn create_file<T, P, B>(fs: &T, path: P, buf: B) -> io::Result<()>
where
    T: FileSystem,
    P: AsRef<Path>,
    B: AsRef<[u8]>,
{
    let opts = OpenOptions::new().write(true).create_new(true);
    let mut writer = fs.open_with_options(path, &opts)?;
    writer.write_all(buf.as_ref())
}

// Used to be part of the public API.
// Keep around for the tests.
fn write_file<T, P, B>(fs: &T, path: P, buf: B) -> io::Result<()>
where
    T: FileSystem,
    P: AsRef<Path>,
    B: AsRef<[u8]>
{
    let mut writer = fs.create(path)?;
    writer.write_all(buf.as_ref())
}

// Used to be part of the public API.
// Keep around for the tests.
fn overwrite_file<T, P, B>(fs: &T, path: P, buf: B) -> io::Result<()>
where
    T: FileSystem,
    P: AsRef<Path>,
    B: AsRef<[u8]>
{
    let opts = OpenOptions::new().write(true).truncate(true);
    let mut writer = fs.open_with_options(path, &opts)?;
    writer.write_all(buf.as_ref())
}

// Used to be part of the public API.
// Keep around for the tests.
fn set_readonly<T: FileSystem, P: AsRef<Path>>(fs: &T, path: P, readonly: bool) -> io::Result<()>
{
    let mut p = fs.metadata(&path)?.permissions();
    p.set_readonly(readonly);
    fs.set_permissions(&path, p)
}

// Used to be part of the public API.
// Keep around for the tests.
fn readonly<P: AsRef<Path>, T: FileSystem>(fs: &T, path: P) -> io::Result<bool>
{
    Ok(fs.metadata(&path)?.permissions().readonly())
}

// Used to be part of the public API.
// Keep around for the tests.
#[cfg(unix)]
fn set_mode<P: AsRef<Path>, T: FileSystem>(fs: &T, path: P, mode: u32) -> io::Result<()> {
    let mut perms = fs.metadata(&path)?.permissions();
    perms.set_mode(mode);
    fs.set_permissions(&path, perms)
}

// Used to be part of the public API.
// Keep around for the tests.
#[cfg(unix)]
fn mode<P: AsRef<Path>, T: FileSystem>(fs: &T, path: P) -> io::Result<u32> {
    Ok(fs.metadata(&path)?.permissions().mode())
}

fn set_current_dir_fails_if_node_does_not_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("does_not_exist");

    let result = fs.set_current_dir(path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn set_current_dir_fails_if_node_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    create_file(fs, &path, "").unwrap();

    let result = fs.set_current_dir(path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn is_dir_returns_true_if_node_is_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    assert!(fs.is_dir(&path));
}

fn is_dir_returns_false_if_node_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    create_file(fs, &path, "").unwrap();

    assert!(!fs.is_dir(&path));
}

fn is_dir_returns_false_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    assert!(!fs.is_dir(parent.join("does_not_exist")));
}

fn is_file_returns_true_if_node_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");

    create_file(fs, &path, "").unwrap();

    assert!(fs.is_file(&path));
}

fn is_file_returns_false_if_node_is_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    assert!(!fs.is_file(&path));
}

fn is_file_returns_false_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    assert!(!fs.is_file(parent.join("does_not_exist")));
}

fn create_dir_creates_new_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    let result = fs.create_dir(&path);

    assert!(result.is_ok());
    assert!(fs.is_dir(path));
}

fn create_dir_fails_if_dir_already_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_dir");

    fs.create_dir(&path).unwrap();

    let result = fs.create_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::AlreadyExists);
}

fn create_dir_fails_if_parent_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("parent/new_dir");

    let result = fs.create_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn create_dir_all_creates_dirs_in_path<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.create_dir_all(parent.join("a/b/c"));

    assert!(result.is_ok());
    assert!(fs.is_dir(parent.join("a")));
    assert!(fs.is_dir(parent.join("a/b")));
    assert!(fs.is_dir(parent.join("a/b/c")));
}

fn create_dir_all_still_succeeds_if_any_dir_already_exists<T: FileSystem>(fs: &T, parent: &Path) {
    fs.create_dir_all(parent.join("a/b")).unwrap();

    let result = fs.create_dir_all(parent.join("a/b/c"));

    assert!(result.is_ok());
    assert!(fs.is_dir(parent.join("a")));
    assert!(fs.is_dir(parent.join("a/b")));
    assert!(fs.is_dir(parent.join("a/b/c")));
}

fn remove_dir_deletes_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");

    fs.create_dir(&path).unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&path));
}

fn remove_dir_does_not_affect_parent<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("parent/child");

    fs.create_dir_all(&path).unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_ok());
    assert!(fs.is_dir(parent.join("parent")));
    assert!(!fs.is_dir(parent.join("child")));
}

fn remove_dir_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.remove_dir(parent.join("does_not_exist"));

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn remove_dir_fails_if_node_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    create_file(fs, &path, "").unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_file(&path));
}

fn remove_dir_fails_if_dir_is_not_empty<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");
    let child = path.join("file");

    fs.create_dir(&path).unwrap();
    create_file(fs, &child, "").unwrap();

    let result = fs.remove_dir(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_dir(&path));
    assert!(fs.is_file(&child));
}

fn remove_dir_all_removes_dir_and_contents<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("dir");
    let child = path.join("file");

    fs.create_dir(&path).unwrap();
    create_file(fs, &child, "").unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&path));
    assert!(!fs.is_file(&child));
    assert!(fs.is_dir(parent));
}

fn remove_dir_all_fails_if_node_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    create_file(fs, &path, "").unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
    assert!(fs.is_file(&path));
}

#[cfg(unix)]
fn remove_dir_all_removes_dir_and_contents_if_descendant_not_writable<
    T: FileSystem,
>(
    fs: &T,
    parent: &Path,
) {
    let mode = 0o555;

    let path = parent.join("dir");
    let child = path.join("child");

    fs.create_dir(&path).unwrap();
    fs.create_dir(&child).unwrap();

    set_mode(fs, &child, mode).unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&path));
    assert!(!fs.is_dir(&child));
}

#[cfg(unix)]
fn remove_dir_all_removes_dir_and_contents_if_descendant_not_executable<
    T: FileSystem,
>(
    fs: &T,
    parent: &Path,
) {
    let mode = 0o666;

    let path = parent.join("dir");
    let child = path.join("child");

    fs.create_dir(&path).unwrap();
    fs.create_dir(&child).unwrap();

    set_mode(fs, &child, mode).unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&path));
    assert!(!fs.is_dir(&child));
}

#[cfg(unix)]
fn remove_dir_all_fails_if_descendant_not_readable<T: FileSystem>(
    fs: &T,
    parent: &Path,
) {
    let mode = 0o333;

    let path = parent.join("dir");
    let child = path.join("child");

    fs.create_dir(&path).unwrap();
    fs.create_dir(&child).unwrap();

    set_mode(fs, &child, mode).unwrap();

    let result = fs.remove_dir_all(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
    assert!(fs.is_dir(&path));
    assert!(fs.is_dir(&child));
}

fn read_dir_returns_dir_entries<T: FileSystem>(fs: &T, parent: &Path) {
    let file1 = parent.join("file1");
    let file2 = parent.join("file2");
    let dir1 = parent.join("dir1");
    let dir2 = parent.join("dir2");
    let file3 = dir1.join("file3");
    let file4 = dir2.join("file4");

    create_file(fs, &file1, "").unwrap();
    create_file(fs, &file2, "").unwrap();
    fs.create_dir(&dir1).unwrap();
    fs.create_dir(&dir2).unwrap();
    create_file(fs, &file3, "").unwrap();
    create_file(fs, &file4, "").unwrap();

    let result = fs.read_dir(parent);

    assert!(result.is_ok());

    let mut entries: Vec<PathBuf> = result.unwrap().map(|e| e.unwrap().path()).collect();
    let expected_paths = &mut [file1, file2, dir1, dir2];

    entries.sort();
    expected_paths.sort();

    assert_eq!(&entries, expected_paths);
}

fn read_dir_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("does_not_exist");
    let result = fs.read_dir(&path);

    assert!(result.is_err());

    match result {
        Ok(_) => panic!("should be an err"),
        Err(err) => assert_eq!(err.kind(), ErrorKind::NotFound),
    }
}

fn read_dir_fails_if_node_is_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    create_file(fs, &path, "").unwrap();

    let result = fs.read_dir(&path);

    assert!(result.is_err());
    match result {
        Ok(_) => panic!("should be an err"),
        Err(err) => assert_eq!(err.kind(), ErrorKind::Other),
    }
}

fn create_object_writes_to_new_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");
    let mut writer = fs.create(&path).unwrap();
    let result = writer.write_all(b"new contents");

    assert!(result.is_ok());

    let contents = read_file(fs, path).unwrap();

    assert_eq!(&contents, b"new contents");
}

fn create_object_fails_if_file_is_readonly<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "").unwrap();
    set_readonly(fs, &path, true).unwrap();

    let result = fs.create(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}

fn write_file_writes_to_new_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");
    let result = write_file(fs, &path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(read_file(fs, path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn write_file_overwrites_contents_of_existing_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    write_file(fs, &path, "old contents").unwrap();

    let result = write_file(fs, &path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(read_file(fs, path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn write_file_fails_if_file_is_readonly<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "").unwrap();
    set_readonly(fs, &path, true).unwrap();

    let result = write_file(fs, &path, "test contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}

fn write_file_fails_if_node_is_a_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_dir");

    fs.create_dir(&path).unwrap();

    let result = write_file(fs, &path, "test contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn overwrite_file_overwrites_contents_of_existing_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    write_file(fs, &path, "old contents").unwrap();

    let result = overwrite_file(fs, &path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(read_file(fs, path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn overwrite_file_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("new_file");
    let result = overwrite_file(fs, &path, "new contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn overwrite_file_fails_if_file_is_readonly<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "").unwrap();
    set_readonly(fs, &path, true).unwrap();

    let result = overwrite_file(fs, &path, "test contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}

fn overwrite_file_fails_if_node_is_a_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_dir");

    fs.create_dir(&path).unwrap();

    let result = overwrite_file(fs, &path, "test contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn read_file_returns_contents_as_bytes<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    write_file(fs, &path, "test text").unwrap();

    let result = read_file(fs, &path);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), br"test text");
}

fn read_file_fails_if_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let result = read_file(fs, &path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn read_file_to_string_returns_contents_as_string<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    write_file(fs, &path, "test text").unwrap();

    let result = read_file_to_string(fs, &path);

    assert!(result.is_ok());
    assert_eq!(&result.unwrap(), "test text");
}

fn read_file_to_string_fails_if_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let result = read_file_to_string(fs, &path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn read_file_to_string_fails_if_contents_are_not_utf8<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    write_file(fs, &path, &[0, 159, 146, 150]).unwrap();

    let result = read_file_to_string(fs, &path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);
}

fn read_file_into_writes_bytes_to_buffer<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let text = "test text";

    write_file(fs, &path, text).unwrap();
    let mut buf = Vec::new();

    let result = read_file_into(fs, &path, &mut buf);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), text.as_bytes().len());
    assert_eq!(buf, br"test text");
}

fn read_file_into_fails_if_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    let result = read_file_into(fs, &path, &mut Vec::new());

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn open_object_writes_bytes_to_buffer<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let text = "test text";

    write_file(fs, &path, text).unwrap();
    let mut buf = Vec::new();

    let mut reader = fs.open(&path).unwrap();
    let result = reader.read_to_end(&mut buf);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), text.as_bytes().len());
    assert_eq!(buf, br"test text");
}

fn open_object_fails_if_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    let result = fs.open(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn create_file_writes_to_new_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");
    let result = create_file(fs, &path, "new contents");

    assert!(result.is_ok());

    let contents = String::from_utf8(read_file(fs, path).unwrap()).unwrap();

    assert_eq!(&contents, "new contents");
}

fn create_file_fails_if_file_already_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "contents").unwrap();

    let result = create_file(fs, &path, "new contents");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::AlreadyExists);
}

fn remove_file_removes_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "").unwrap();

    let result = fs.remove_file(&path);

    assert!(result.is_ok());

    let result = read_file(fs, &path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn remove_file_fails_if_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = fs.remove_file(parent.join("does_not_exist"));

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn remove_file_fails_if_node_is_a_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_dir");

    fs.create_dir(&path).unwrap();

    let result = fs.remove_file(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn copy_file_copies_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    create_file(fs, &from, "test").unwrap();

    let result = fs.copy_file(&from, &to);

    assert!(result.is_ok());

    let result = read_file(fs, &to);

    assert!(result.is_ok());
    assert_eq!(&result.unwrap(), b"test");
}

fn copy_file_overwrites_destination_file<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    create_file(fs, &from, "expected").unwrap();
    create_file(fs, &to, "should be overwritten").unwrap();

    let result = fs.copy_file(&from, &to);

    assert!(result.is_ok());

    let result = read_file(fs, &to);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), b"expected");
}

fn copy_file_fails_if_original_file_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    let result = fs.copy_file(&from, &to);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
    assert!(!fs.is_file(&to));
}

fn copy_file_fails_if_destination_file_is_readonly<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    create_file(fs, &from, "test").unwrap();
    create_file(fs, &to, "").unwrap();
    set_readonly(fs, &to, true).unwrap();

    let result = fs.copy_file(&from, &to);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
}

fn copy_file_fails_if_original_node_is_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    fs.create_dir(&from).unwrap();

    let result = fs.copy_file(&from, &to);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);
}

fn copy_file_fails_if_destination_node_is_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    create_file(fs, &from, "").unwrap();
    fs.create_dir(&to).unwrap();

    let result = fs.copy_file(&from, &to);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn rename_renames_a_file<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    create_file(fs, &from, "contents").unwrap();

    let result = fs.rename(&from, &to);

    assert!(result.is_ok());
    assert!(!fs.is_file(&from));

    let result = read_file_to_string(fs, &to);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "contents");
}

fn rename_renames_a_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");
    let child = from.join("child");

    fs.create_dir(&from).unwrap();
    create_file(fs, &child, "child").unwrap();

    let result = fs.rename(&from, &to);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&from));

    let result = read_file_to_string(fs, to.join("child"));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "child");
}

fn rename_overwrites_destination_file<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    create_file(fs, &from, "from").unwrap();
    create_file(fs, &to, "to").unwrap();

    let result = fs.rename(&from, &to);

    assert!(result.is_ok());
    assert!(!fs.is_file(&from));

    let result = read_file_to_string(fs, &to);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "from");
}

fn rename_overwrites_empty_destination_directory<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");
    let child = from.join("child");

    fs.create_dir(&from).unwrap();
    fs.create_dir(&to).unwrap();
    create_file(fs, &child, "child").unwrap();

    let result = fs.rename(&from, &to);

    assert!(result.is_ok(), "err: {:?}", result);
    assert!(!fs.is_dir(&from));

    let result = read_file_to_string(fs, to.join("child"));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "child");
}

fn rename_renames_all_descendants<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");
    let child_file = from.join("child_file");
    let child_dir = from.join("child_dir");
    let grandchild = child_dir.join("grandchild");

    fs.create_dir(&from).unwrap();
    create_file(fs, &child_file, "child_file").unwrap();
    fs.create_dir(&child_dir).unwrap();
    create_file(fs, &grandchild, "grandchild").unwrap();

    let result = fs.rename(&from, &to);

    assert!(result.is_ok());
    assert!(!fs.is_dir(&from));

    let result = read_file_to_string(fs, to.join("child_file"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "child_file");

    let result = read_file_to_string(fs, to.join("child_dir").join("grandchild"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "grandchild");
}

fn rename_fails_if_original_path_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");

    let result = fs.rename(&from, &to);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn rename_fails_if_original_and_destination_are_different_types<T: FileSystem>(
    fs: &T,
    parent: &Path,
) {
    let file = parent.join("file");
    let dir = parent.join("dir");

    create_file(fs, &file, "").unwrap();
    fs.create_dir(&dir).unwrap();

    let result = fs.rename(&file, &dir);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);

    let result = fs.rename(&dir, &file);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn rename_fails_if_destination_directory_is_not_empty<T: FileSystem>(fs: &T, parent: &Path) {
    let from = parent.join("from");
    let to = parent.join("to");
    let child = to.join("child");

    fs.create_dir(&from).unwrap();
    fs.create_dir(&to).unwrap();
    create_file(fs, &child, "child").unwrap();

    let result = fs.rename(&from, &to);

    assert!(result.is_err());
}

fn readonly_returns_write_permission<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "").unwrap();

    let result = readonly(fs, &path);

    assert!(result.is_ok());
    assert!(!result.unwrap());

    set_readonly(fs, &path, true).unwrap();

    let result = readonly(fs, &path);

    assert!(result.is_ok());
    assert!(result.unwrap());
}

fn readonly_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = readonly(fs, parent.join("does_not_exist"));

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn set_readonly_toggles_write_permission_of_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_file");

    create_file(fs, &path, "").unwrap();

    let result = set_readonly(fs, &path, true);

    assert!(result.is_ok());
    assert!(write_file(fs, &path, "readonly").is_err());

    let result = set_readonly(fs, &path, false);

    assert!(result.is_ok());
    assert!(write_file(fs, &path, "no longer readonly").is_ok());
}

fn set_readonly_toggles_write_permission_of_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test_dir");

    fs.create_dir(&path).unwrap();

    let result = set_readonly(fs, &path, true);

    assert!(result.is_ok());
    assert!(write_file(fs, &path.join("file"), "").is_err());

    let result = set_readonly(fs, &path, false);

    assert!(result.is_ok());
    assert!(write_file(fs, &path.join("file"), "").is_ok());
}

fn set_readonly_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = set_readonly(fs, parent.join("does_not_exist"), true);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);

    let result = set_readonly(fs, parent.join("does_not_exist"), true);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn len_returns_size_of_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");
    let result = create_file(fs, &path, "");

    assert!(result.is_ok());

    let len = fs.open(&path).unwrap().metadata().unwrap().len();

    assert_eq!(len, 0);

    let result = write_file(fs, &path, "contents");

    assert!(result.is_ok());

    let len = fs.open(&path).unwrap().metadata().unwrap().len();

    assert_eq!(len, 8);
}

fn open_objects_read_independently<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();

    let mut readers = (fs.open(&path).unwrap(), fs.open(path).unwrap());
    let mut bufs = (vec![], vec![]);
    readers.0.read_to_end(&mut bufs.0).unwrap();
    readers.1.read_to_end(&mut bufs.1).unwrap();
    assert_eq!(bufs.0, b"test text");
    assert_eq!(bufs.1, b"test text");
}

fn open_object_cannot_open_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    let reader = fs.open(&dir);
    assert!(reader.is_err());
    assert_eq!(reader.unwrap_err().kind(), ErrorKind::NotFound);
}

fn open_object_read_returns_length<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();

    let mut buf = vec![];
    let result = reader.read_to_end(&mut buf);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 9);
}

fn open_object_reads_chunked<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();

    let mut buf = vec![0; 5];
    reader.read_exact(&mut buf).unwrap();
    assert_eq!(buf, b"test ");

    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"text");
}

fn open_object_reads_ok_after_file_deleted<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();
    fs.remove_file(&path).unwrap();
    // verify file is really gone
    let result = read_file(fs, &path);
    assert!(result.is_err());
    // check that reader can still read it
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"test text");
}

fn open_object_reads_ok_after_file_overwritten<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();
    fs.remove_file(&path).unwrap();
    write_file(fs, &path, b"the quick brown fox").unwrap();
    // check that reader still sees the old contents
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"test text");
}

fn open_object_reads_ok_after_parent_dir_deleted<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();
    fs.remove_dir_all(&dir).unwrap();
    // verify file is really gone
    let result = read_file(fs, &path);
    assert!(result.is_err());
    // check that reader can still read it
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"test text");
}

fn open_object_reads_ok_after_file_renamed<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();
    let renamed_path = parent.join("test.html");
    fs.rename(&path, &renamed_path).unwrap();
    // verify file is really renamed
    let result = read_file(fs, &path);
    assert!(result.is_err());
    let result = read_file(fs, &renamed_path);
    assert!(result.is_ok());
    // check that reader can still read it with the reader
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"test text");
}

fn open_object_reads_ok_after_parent_dir_renamed<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();
    let renamed_dir = parent.join("test2");
    fs.rename(&dir, &renamed_dir).unwrap();
    // verify file is really gone
    let result = read_file(fs, &path);
    assert!(result.is_err());
    // check that reader can still read it
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"test text");
}

fn open_object_reads_ok_after_parent_dir_moved<T: FileSystem>(fs: &T, parent: &Path) {
    // parent |-> test1 -> test.txt
    //        |-> test2
    // after moving test1:
    // parent |-> test2 -> test1 -> test.txt
    //
    let dir1 = parent.join("test1");
    let dir2 = parent.join("test2");
    let path = dir1.join("test.txt");
    fs.create_dir(&dir1).unwrap();
    fs.create_dir(&dir2).unwrap();
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();

    fs.rename(&dir1, dir2.join("test1")).unwrap();
    // verify that original file is gone
    let result = read_file(fs, path);
    assert!(result.is_err());
    // check that reader can still read the file
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"test text");
}

fn open_object_reads_ok_beyond_eof<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();

    let result = reader.read_to_end(&mut buf);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

fn open_object_reads_ok_after_file_updated<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut reader = fs.open(&path).unwrap();
    let mut buf = vec![0; 5];
    reader.read_exact(&mut buf).unwrap();
    assert_eq!(buf, b"test ");

    write_file(fs, &path, "the quick brown fox").unwrap();
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"uick brown fox");
}

fn open_object_reads_ok_after_file_shrunk<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();
    let mut buf = vec![0; 10];
    reader.read_exact(&mut buf).unwrap();
    assert_eq!(buf, b"the quick ");

    write_file(fs, &path, "test").unwrap();
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"");
}

fn open_object_can_seek_from_start_then_read<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();

    let result = reader.seek(SeekFrom::Start(5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);

    let result = reader.seek(SeekFrom::Start(5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);

    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"uick brown fox");
}

fn open_object_can_seek_from_current_then_read<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();

    let result = reader.seek(SeekFrom::Current(5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);

    let result = reader.seek(SeekFrom::Current(5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 10);

    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"brown fox");
}

fn open_object_can_seek_from_end_then_read<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let msg = b"the quick brown fox";
    write_file(fs, &path, msg).unwrap();
    let mut reader = fs.open(&path).unwrap();

    let result = reader.seek(SeekFrom::End(-5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap() as usize, msg.len() - 5);

    let result = reader.seek(SeekFrom::End(-5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap() as usize, msg.len() - 5);

    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"n fox");
}

fn open_object_fails_if_seeks_before_byte_0<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();

    reader.seek(SeekFrom::Start(5)).unwrap();

    let result = reader.seek(SeekFrom::Current(-55));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidInput);

    // verify that the error did not change the position
    let current_pos = reader.seek(SeekFrom::Current(0)).unwrap();
    assert_eq!(current_pos, 5);
}

fn open_object_can_seek_and_read_beyond_eof<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"the quick brown fox").unwrap();
    let mut reader = fs.open(&path).unwrap();

    let result = reader.seek(SeekFrom::Current(55));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 55);

    let mut buf = vec![];
    let result = reader.read_to_end(&mut buf);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

fn create_objects_write_independently<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    let mut writers = (fs.create(&path).unwrap(), fs.create(&path).unwrap());
    let buf = b"the quick brown fox";
    writers.0.write_all(buf).unwrap();
    let read_buf1 = read_file(fs, &path).unwrap();
    writers.1.write_all(buf).unwrap();
    let read_buf2 = read_file(fs, &path).unwrap();
    assert_eq!(read_buf1, read_buf2);
}

fn create_object_cannot_overwrite_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let writer = fs.create(&dir);
    assert!(writer.is_err());
    assert_eq!(writer.unwrap_err().kind(), ErrorKind::Other);
}

fn create_object_writes_chunked<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test").unwrap();
    writer.write_all(b" text").unwrap();
    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"test text");
}

fn create_object_writes_ok_beyond_eof<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    write_file(fs, &path, b"").unwrap();
    writer.write_all(b"test text").unwrap();
    let buf = read_file(fs, &path).unwrap();
    assert_eq!(buf, b"\0\0\0\0\0\0\0\0\0test text");
}

fn create_object_writes_ok_after_file_deleted<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    fs.remove_file(&path).unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());
}

fn create_object_writes_ok_after_file_overwritten<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    write_file(fs, &path, b"the quick brown fox").unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());
}

fn create_object_writes_ok_after_parent_dir_deleted<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    let path = dir.join("test.txt");
    fs.create_dir(&dir).unwrap();
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    fs.remove_dir_all(&dir).unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());
}

fn create_object_writes_ok_after_file_renamed<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let renamed_path = parent.join("test.html");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    fs.rename(&path, &renamed_path).unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());

    let contents = read_file(fs, &renamed_path).unwrap();
    assert_eq!(contents, b"test texttest text");
}

fn create_object_writes_ok_after_parent_dir_renamed<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    let renamed_dir = parent.join("test2");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    fs.rename(&dir, &renamed_dir).unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());

    let contents = read_file(fs, renamed_dir.join("test.txt")).unwrap();
    assert_eq!(contents, b"test texttest text");
}

fn create_object_writes_ok_after_parent_dir_moved<T: FileSystem>(fs: &T, parent: &Path) {
    // parent |-> test1 -> test.txt
    //        |-> test2
    // after moving test1:
    // parent |-> test2 -> test1 -> test.txt
    //
    let dir1 = parent.join("test1");
    let dir2 = parent.join("test2");
    let path = dir1.join("test.txt");
    fs.create_dir(&dir1).unwrap();
    fs.create_dir(&dir2).unwrap();
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    let new_root = dir2.join("test1");
    fs.rename(&dir1, &new_root).unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());

    let contents = read_file(fs, new_root.join("test.txt")).unwrap();
    assert_eq!(contents, b"test texttest text");
}

fn create_object_writes_ok_after_file_updated_long<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    write_file(fs, &path, b"the quick brown fox").unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"the quicktest textx");
}

fn create_object_writes_ok_after_file_updated_short<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    write_file(fs, &path, b"the quick brown").unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"the quicktest text");
}

fn create_object_writes_ok_after_file_shrunk<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    write_file(fs, &path, b"hello").unwrap();
    let result = writer.write_all(b"test text");
    assert!(result.is_ok());

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"hello\0\0\0\0test text");
}

fn create_object_can_seek_then_overwrite<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"the quick brown fox").unwrap();

    writer.seek(SeekFrom::Start(5)).unwrap();
    let cur = writer.seek(SeekFrom::Current(0)).unwrap();
    assert_eq!(cur, 5);

    let result = writer.write_all(b"hello");
    assert!(result.is_ok());

    let buf = read_file(fs, &path).unwrap();
    assert_eq!(buf, b"the qhellobrown fox");
}

fn create_object_can_seek_then_overwrite_and_extend<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    writer.seek(SeekFrom::Start(5)).unwrap();
    let cur = writer.seek(SeekFrom::Current(0)).unwrap();
    assert_eq!(cur, 5);

    let result = writer.write_all(b"the quick brown fox");
    assert!(result.is_ok());

    let buf = read_file(fs, &path).unwrap();
    assert_eq!(buf, b"test the quick brown fox");
}

fn create_object_can_seek_then_extend<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    writer.seek(SeekFrom::Start(12)).unwrap();
    let cur = writer.seek(SeekFrom::Current(0)).unwrap();
    assert_eq!(cur, 12);

    let result = writer.write_all(b"test");
    assert!(result.is_ok());

    let buf = read_file(fs, &path).unwrap();
    assert_eq!(buf, b"test text\0\0\0test");
}

fn open_object_cannot_write<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    create_file(fs, &path, vec![]).unwrap();

    let mut reader = fs.open(&path).unwrap();
    let result = reader.write(b"the quick brown fox");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn create_object_cannot_read<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");

    let mut writer = fs.create(&path).unwrap();
    let mut buf = vec![];
    let result = writer.read_to_end(&mut buf);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Other);
}

fn set_len_on_create_object_truncates_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let writer = fs.create(&path).unwrap();
    write_file(fs, &path, b"test text").unwrap();

    let result = writer.set_len(4);
    assert!(result.is_ok());

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"test");
}

fn set_len_on_create_object_extends_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let writer = fs.create(&path).unwrap();
    write_file(fs, &path, b"test").unwrap();

    let result = writer.set_len(9);
    assert!(result.is_ok());

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"test\0\0\0\0\0");
}

fn set_len_on_create_object_doesnt_change_cursor<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    write_file(fs, &path, b"test").unwrap();

    let result = writer.set_len(9);
    assert!(result.is_ok());

    let pos = writer.seek(SeekFrom::Current(0)).unwrap();
    assert_eq!(pos, 0);
}

fn fs_dir_metadata_is_dir<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test");
    fs.create_dir(&path).unwrap();

    let md = fs.metadata(&path).unwrap();
    assert!(!md.is_file());
    assert!(md.is_dir());
}

fn fs_dir_metadata_has_correct_len<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    fs.create_dir(&path).unwrap();

    let md = fs.metadata(&path).unwrap();
    // to keep things portable, don't test for a particular value
    assert_ne!(md.len(), 0);
}

fn fs_file_metadata_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();

    let md = fs.metadata(&path).unwrap();
    assert!(md.is_file());
    assert!(!md.is_dir());
}

fn fs_file_metadata_has_correct_len<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();

    let md = fs.metadata(&path).unwrap();
    assert_eq!(md.len(), 9);
}

fn fs_file_metadata_len_is_immutable<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let md = fs.metadata(&path).unwrap();

    assert_eq!(md.len(), 9);

    write_file(fs, &path, b"hi").unwrap();
    assert_eq!(md.len(), 9);
}

fn fs_file_metadata_fails_if_file_doesn_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("does_not_exist");
    let result = fs.metadata(&path);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn open_object_metadata_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let reader = fs.open(&path).unwrap();

    let md = reader.metadata().unwrap();
    assert!(md.is_file());
    assert!(!md.is_dir());
}

fn open_object_metadata_has_correct_len<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let reader = fs.open(&path).unwrap();

    let md = reader.metadata().unwrap();
    assert_eq!(md.len(), 9);
}

fn open_object_metadata_len_is_immutable<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let reader = fs.open(&path).unwrap();
    let md = reader.metadata().unwrap();

    assert_eq!(md.len(), 9);

    write_file(fs, &path, b"hi").unwrap();
    assert_eq!(md.len(), 9);
}

fn create_object_metadata_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let writer = fs.create(&path).unwrap();

    let md = writer.metadata().unwrap();
    assert!(md.is_file());
    assert!(!md.is_dir());
}

fn create_object_metadata_has_correct_len<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();

    let md = writer.metadata().unwrap();
    assert_eq!(md.len(), 9);
}

fn create_object_metadata_len_is_immutable<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let mut writer = fs.create(&path).unwrap();
    writer.write_all(b"test text").unwrap();
    let md = writer.metadata().unwrap();

    assert_eq!(md.len(), 9);

    writer.write_all(b"hi").unwrap();
    assert_eq!(md.len(), 9);
}

fn open_writable<T: FileSystem>(fs: &T, path: &Path) -> io::Result<T::File> {
    let opts = OpenOptions::new().write(true);
    fs.open_with_options(path, &opts)
}

fn writable_object_does_not_create_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let result = open_writable(fs, &path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn writable_object_sets_cursor_to_beginning<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut writer = open_writable(fs, &path).unwrap();
    let pos = writer.seek(SeekFrom::Current(0)).unwrap();
    assert_eq!(pos, 0);
}

fn writable_object_allows_append<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut writer = open_writable(fs, &path).unwrap();
    writer.seek(SeekFrom::End(0)).unwrap();

    writer.write_all(b"hello").unwrap();

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"test texthello");
}

fn writable_object_truncates<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut writer = open_writable(fs, &path).unwrap();
    writer.seek(SeekFrom::End(-4)).unwrap();

    writer.write_all(b"hello").unwrap();

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(String::from_utf8(contents).unwrap(), "test hello");
}

fn writable_object_allows_write_short<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut writer = open_writable(fs, &path).unwrap();

    writer.write_all(b"hello").unwrap();

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"hellotext");
}

fn writable_object_allows_write_long<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut writer = open_writable(fs, &path).unwrap();

    writer.write_all(b"the quick brown fox").unwrap();

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"the quick brown fox");
}

fn writable_object_extends_file<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, b"test text").unwrap();
    let mut writer = open_writable(fs, &path).unwrap();

    writer.seek(SeekFrom::Start(12)).unwrap();
    writer.write_all(b"hi").unwrap();

    let contents = read_file(fs, &path).unwrap();
    assert_eq!(contents, b"test text\0\0\0hi");
}

fn canonicalize_ok_if_file_exists<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    write_file(fs, &path, "test.txt").unwrap();
    let result = fs.canonicalize(&path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), path);
}

fn canonicalize_ok_if_root<T: FileSystem>(fs: &T, _parent: &Path) {
    let path = PathBuf::from(std::path::MAIN_SEPARATOR.to_string());
    let result = fs.canonicalize(&path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), path);
}

fn canonicalize_fails_if_empty<T: FileSystem>(fs: &T, _parent: &Path) {
    let path = PathBuf::from("");
    let result = fs.canonicalize(&path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn canonicalize_dot_is_current_dir<T: FileSystem>(fs: &T, _parent: &Path) {
    let path = PathBuf::from(".");
    let result = fs.canonicalize(&path);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), fs.current_dir().unwrap());
}

fn canonicalize_ok_if_relative_path<T: FileSystem>(fs: &T, parent: &Path) {
    let save_current_dir = fs.current_dir().unwrap();

    fs.set_current_dir(&parent).unwrap();
    let result = fs.canonicalize(&PathBuf::from("."));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), parent);

    fs.set_current_dir(save_current_dir).unwrap();
}

fn canonicalize_ok_if_path_ends_in_dotdot<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();

    let dotdot = dir.join("..");
    let result = fs.canonicalize(&dotdot);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), parent);
}

fn canonicalize_fails_if_file_doesnt_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("test.txt");
    let result = fs.canonicalize(&path);
    assert!(result.is_err());
}

fn canonicalize_ok_with_dotdot_if_paths_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    write_file(fs, &path, "test text").unwrap();

    let dotdot = dir.join("..").join("test").join("test.txt");
    let result = fs.canonicalize(&dotdot);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), path);
}

fn canonicalize_fails_with_dotdot_if_path_doesnt_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    write_file(fs, &path, "test text").unwrap();

    let dotdot = dir.join("does_not_exist").join("..").join("test.txt");
    let result = fs.canonicalize(&dotdot);
    assert!(result.is_err());
}

fn canonicalize_cant_go_lower_than_root<T: FileSystem>(fs: &T, parent: &Path) {
    let num_dirs = parent.iter().count();
    let dotdot_root: PathBuf = std::iter::repeat("..").take(num_dirs * 2)
                        .collect();
    let root = parent.iter().nth(0).unwrap();
    let result = fs.canonicalize(&dotdot_root);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), root);
}

#[cfg(not(target_os = "macos"))]
fn canonicalize_fails_if_subpath_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    write_file(fs, &path, "test text").unwrap();

    let dotdot = parent.join("test/test.txt/../test.txt");
    let result = fs.canonicalize(&dotdot);
    assert!(result.is_err());
}

#[cfg(target_os = "macos")]
fn canonicalize_ok_if_subpath_is_file<T: FileSystem>(fs: &T, parent: &Path) {
    let dir = parent.join("test");
    fs.create_dir(&dir).unwrap();
    let path = dir.join("test.txt");
    write_file(fs, &path, "content 3").unwrap();

    let dotdot = parent.join("test/test.txt/../test.txt");
    let result = fs.canonicalize(&dotdot);
    assert!(result.is_ok());

    let content = read_file(fs, result.unwrap().as_path());
    assert_eq!(content.unwrap(), b"content 3");

}

#[cfg(unix)]
fn mode_returns_permissions<T: FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    create_file(fs, &path, "").unwrap();
    set_mode(fs, &path, 0o644).unwrap();

    let result = mode(fs, &path);

    assert!(result.is_ok());
    assert_eq!(result.unwrap() % 0o100_000, 0o644);

    set_mode(fs, &path, 0o600).unwrap();

    let result = mode(fs, &path);

    assert!(result.is_ok());
    assert_eq!(result.unwrap() % 0o100_000, 0o600);

    set_readonly(fs, &path, true).unwrap();

    let result = mode(fs, &path);

    assert!(result.is_ok());
    assert_eq!(result.unwrap() % 0o100_000, 0o400);
}

#[cfg(unix)]
fn mode_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = mode(fs, parent.join("does_not_exist"));

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

#[cfg(unix)]
fn set_mode_sets_permissions<T: FileSystem + FileSystem>(fs: &T, parent: &Path) {
    let path = parent.join("file");

    create_file(fs, &path, "").unwrap();

    let result = set_mode(fs, &path, 0o000);

    assert!(result.is_ok());

    let readonly_result = readonly(fs, &path);

    assert!(readonly_result.is_ok());
    assert!(readonly_result.unwrap());

    let read_result = read_file(fs, &path);
    let write_result = write_file(fs, &path, "should not be allowed");

    assert!(read_result.is_err());
    assert!(write_result.is_err());
    assert_eq!(read_result.unwrap_err().kind(), ErrorKind::PermissionDenied);
    assert_eq!(
        write_result.unwrap_err().kind(),
        ErrorKind::PermissionDenied
    );

    let result = set_mode(fs, &path, 0o200);

    assert!(result.is_ok());

    let read_result = read_file(fs, &path);
    let write_result = write_file(fs, &path, "should be allowed");

    assert!(read_result.is_err());
    assert!(write_result.is_ok());
    assert_eq!(read_result.unwrap_err().kind(), ErrorKind::PermissionDenied);

    let readonly_result = readonly(fs, &path);

    assert!(readonly_result.is_ok());
    assert!(!readonly_result.unwrap());

    let result = set_mode(fs, &path, 0o644);

    assert!(result.is_ok());

    let readonly_result = readonly(fs, &path);

    assert!(readonly_result.is_ok());
    assert!(!readonly_result.unwrap());
}

#[cfg(unix)]
fn set_mode_fails_if_node_does_not_exist<T: FileSystem>(fs: &T, parent: &Path) {
    let result = set_mode(fs, parent.join("does_not_exist"), 0o644);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
}

fn temp_dir_creates_tempdir<T: FileSystem + TempFileSystem>(fs: &T, _: &Path) {
    let path = {
        let result = fs.temp_dir("test");

        assert!(result.is_ok());

        let temp_dir = result.unwrap();

        assert!(fs.is_dir(temp_dir.path()));

        temp_dir.path().to_path_buf()
    };

    assert!(!fs.is_dir(&path));
    assert!(fs.is_dir(path.parent().unwrap()));
}

fn temp_dir_creates_unique_dir<T: FileSystem + TempFileSystem>(fs: &T, _: &Path) {
    let first = fs.temp_dir("test").unwrap();
    let second = fs.temp_dir("test").unwrap();

    assert_ne!(first.path(), second.path());
}
