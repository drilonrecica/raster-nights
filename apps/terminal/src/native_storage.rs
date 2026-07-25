// SPDX-License-Identifier: MPL-2.0

//! Native filesystem implementation of the shared byte-storage boundary.

use std::{
    env,
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use raster_storage::{ByteStorage, StorageError, StorageKey};

const DATA_DIRECTORY_OVERRIDE: &str = "RASTER_NIGHTS_DATA_DIR";
static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Filesystem-backed storage for the native host.
#[derive(Clone, Debug)]
pub(crate) struct NativeByteStorage {
    root: PathBuf,
}

impl NativeByteStorage {
    /// Resolve and prepare the current user's Raster Nights data directory.
    pub(crate) fn open() -> Result<Self, StorageError> {
        Self::at(resolve_data_directory()?)
    }

    fn at(root: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&root)
            .map_err(|error| storage_error("create data directory", &root, &error))?;
        Ok(Self { root })
    }

    fn path(&self, key: StorageKey) -> PathBuf {
        self.root.join(key.file_name())
    }
}

impl ByteStorage for NativeByteStorage {
    fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, StorageError> {
        let path = self.path(key);
        match fs::read(&path) {
            Ok(data) => Ok(Some(data)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(storage_error("read", &path, &error)),
        }
    }

    fn write(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError> {
        atomic_write(&self.root, &self.path(key), data)
    }

    fn remove(&mut self, key: StorageKey) -> Result<(), StorageError> {
        let path = self.path(key);
        match fs::remove_file(&path) {
            Ok(()) => sync_directory(&self.root),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(storage_error("remove", &path, &error)),
        }
    }

    fn preserve_corrupt(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError> {
        let directory = self.root.join("diagnostics").join("corrupt");
        fs::create_dir_all(&directory)
            .map_err(|error| storage_error("create corrupt-data directory", &directory, &error))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let file_name = format!(
            "{}.{timestamp}-{}-{sequence}.corrupt",
            key.file_name(),
            std::process::id()
        );
        atomic_write(&directory, &directory.join(file_name), data)
    }
}

fn resolve_data_directory() -> Result<PathBuf, StorageError> {
    resolve_data_directory_from(
        env::var_os(DATA_DIRECTORY_OVERRIDE),
        env::var_os("HOME"),
        env::var_os("XDG_DATA_HOME"),
    )
}

fn resolve_data_directory_from(
    override_path: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
    xdg_data_home: Option<std::ffi::OsString>,
) -> Result<PathBuf, StorageError> {
    if let Some(override_path) = override_path {
        if override_path.is_empty() {
            return Err(StorageError::new(format!(
                "{DATA_DIRECTORY_OVERRIDE} is set but empty"
            )));
        }
        return Ok(PathBuf::from(override_path));
    }

    resolve_platform_data_directory(home, xdg_data_home)
}

#[cfg(target_os = "macos")]
fn resolve_platform_data_directory(
    home: Option<std::ffi::OsString>,
    _xdg_data_home: Option<std::ffi::OsString>,
) -> Result<PathBuf, StorageError> {
    home.filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| path.join("Library/Application Support/Raster Nights"))
        .ok_or_else(|| {
            StorageError::new(
                "cannot determine the Raster Nights data directory: HOME is unavailable",
            )
        })
}

#[cfg(not(target_os = "macos"))]
fn resolve_platform_data_directory(
    home: Option<std::ffi::OsString>,
    xdg_data_home: Option<std::ffi::OsString>,
) -> Result<PathBuf, StorageError> {
    if let Some(path) = xdg_data_home
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Ok(path.join("raster-nights"));
    }

    home.filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| path.join(".local/share/raster-nights"))
        .ok_or_else(|| {
            StorageError::new(
                "cannot determine the Raster Nights data directory: \
                 XDG_DATA_HOME is not absolute and HOME is unavailable",
            )
        })
}

fn atomic_write(directory: &Path, destination: &Path, data: &[u8]) -> Result<(), StorageError> {
    fs::create_dir_all(directory)
        .map_err(|error| storage_error("create parent directory", directory, &error))?;

    let (temporary_path, mut temporary_file) = create_temporary_sibling(destination)?;
    let result = (|| {
        temporary_file
            .write_all(data)
            .map_err(|error| storage_error("write temporary file", &temporary_path, &error))?;
        temporary_file
            .flush()
            .map_err(|error| storage_error("flush temporary file", &temporary_path, &error))?;
        temporary_file
            .sync_all()
            .map_err(|error| storage_error("sync temporary file", &temporary_path, &error))?;
        drop(temporary_file);

        fs::rename(&temporary_path, destination)
            .map_err(|error| storage_error("replace data file", destination, &error))?;
        sync_directory(directory)
    })();

    if result.is_err() {
        let _cleanup_result = fs::remove_file(&temporary_path);
    }
    result
}

fn create_temporary_sibling(destination: &Path) -> Result<(PathBuf, File), StorageError> {
    let directory = destination.parent().ok_or_else(|| {
        StorageError::new(format!(
            "data file has no parent directory: {}",
            destination.display()
        ))
    })?;
    let destination_name = destination
        .file_name()
        .unwrap_or_else(|| OsStr::new("data"));

    for _ in 0..32 {
        let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary_name = format!(
            ".{}.{}.{sequence}.tmp",
            destination_name.to_string_lossy(),
            std::process::id()
        );
        let path = directory.join(temporary_name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(storage_error("create temporary file", &path, &error)),
        }
    }

    Err(StorageError::new(format!(
        "could not allocate a temporary sibling for {}",
        destination.display()
    )))
}

fn sync_directory(directory: &Path) -> Result<(), StorageError> {
    File::open(directory)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| storage_error("sync directory", directory, &error))
}

fn storage_error(operation: &str, path: &Path, error: &io::Error) -> StorageError {
    StorageError::new(format!("failed to {operation} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_directory(name: &str) -> PathBuf {
        let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "raster-nights-{name}-{}-{sequence}",
            std::process::id()
        ))
    }

    fn test_storage(name: &str) -> (PathBuf, NativeByteStorage) {
        let directory = unique_test_directory(name);
        let storage =
            NativeByteStorage::at(directory.clone()).expect("test storage should be created");
        (directory, storage)
    }

    #[test]
    fn missing_records_are_absent_and_removal_is_idempotent() {
        let (directory, mut storage) = test_storage("missing");

        assert_eq!(storage.read(StorageKey::Settings).unwrap(), None);
        storage.remove(StorageKey::Settings).unwrap();

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn writes_replace_records_without_leaving_temporary_files() {
        let (directory, mut storage) = test_storage("write");

        storage.write(StorageKey::Scores, b"first").unwrap();
        storage.write(StorageKey::Scores, b"second").unwrap();

        assert_eq!(
            storage.read(StorageKey::Scores).unwrap(),
            Some(b"second".to_vec())
        );
        let entries = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(entries, vec![StorageKey::Scores.file_name()]);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn corrupt_bytes_are_preserved_separately() {
        let (directory, mut storage) = test_storage("corrupt");
        let corrupt = b"not valid structured data";

        storage
            .preserve_corrupt(StorageKey::SystemState, corrupt)
            .unwrap();

        let corrupt_directory = directory.join("diagnostics/corrupt");
        let files = fs::read_dir(&corrupt_directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        assert!(
            files[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("system-state.json.")
        );
        assert_eq!(fs::read(&files[0]).unwrap(), corrupt);

        fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn linux_directory_prefers_absolute_xdg_path() {
        assert_eq!(
            resolve_platform_data_directory(Some("/home/tester".into()), Some("/var/data".into()))
                .unwrap(),
            PathBuf::from("/var/data/raster-nights")
        );
        assert_eq!(
            resolve_platform_data_directory(Some("/home/tester".into()), Some("relative".into()))
                .unwrap(),
            PathBuf::from("/home/tester/.local/share/raster-nights")
        );
    }

    #[test]
    fn explicit_directory_override_takes_precedence() {
        assert_eq!(
            resolve_data_directory_from(
                Some("isolated-data".into()),
                Some("/home/tester".into()),
                Some("/var/data".into()),
            )
            .unwrap(),
            PathBuf::from("isolated-data")
        );
        assert!(
            resolve_data_directory_from(
                Some(std::ffi::OsString::new()),
                Some("/home/tester".into()),
                None,
            )
            .unwrap_err()
            .to_string()
            .contains(DATA_DIRECTORY_OVERRIDE)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_directory_uses_application_support() {
        assert_eq!(
            resolve_platform_data_directory(Some("/Users/tester".into()), None).unwrap(),
            PathBuf::from("/Users/tester/Library/Application Support/Raster Nights")
        );
    }
}
