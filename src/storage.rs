//! Character collection paths and crash-resistant file persistence.

use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

const DEFAULT_CHARACTER_DIRECTORY: &str = ".";
static NEXT_TEMPORARY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub(crate) struct CharacterRepository {
    directory: PathBuf,
}

impl CharacterRepository {
    pub(crate) fn new(directory: Option<&Path>) -> Self {
        Self {
            directory: directory.map_or_else(
                || PathBuf::from(DEFAULT_CHARACTER_DIRECTORY),
                Path::to_path_buf,
            ),
        }
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn resolve(&self, reference: &Path) -> PathBuf {
        if reference.is_file()
            || reference.extension().is_some()
            || reference.components().count() > 1
        {
            return reference.to_path_buf();
        }
        self.directory.join(reference).with_extension("json")
    }

    pub(crate) fn json_paths(&self) -> Result<Vec<PathBuf>, String> {
        if !self.directory.exists() {
            return Ok(Vec::new());
        }
        if !self.directory.is_dir() {
            return Err(format!(
                "character collection is not a directory: {}",
                self.directory.display()
            ));
        }
        let mut paths = fs::read_dir(&self.directory)
            .map_err(|error| format!("unable to read {}: {error}", self.directory.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("unable to read {}: {error}", self.directory.display()))?
            .into_iter()
            .filter(|path| {
                path.is_file()
                    && path
                        .extension()
                        .is_some_and(|extension| extension == "json")
            })
            .collect::<Vec<_>>();
        paths.sort();
        Ok(paths)
    }
}

pub(crate) fn create_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("unable to create {}: {error}", parent.display()))?;
    }
    Ok(())
}

pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    create_parent(path)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("character.json");
    let temporary = parent.join(format!(
        ".{file_name}.tmp-{}-{}",
        std::process::id(),
        NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("unable to create {}: {error}", temporary.display()))?;
        file.write_all(contents)
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("unable to write {}: {error}", temporary.display()))?;
        replace(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(windows))]
fn replace(source: &Path, destination: &Path) -> Result<(), String> {
    fs::rename(source, destination).map_err(|error| {
        format!(
            "unable to replace {} with {}: {error}",
            destination.display(),
            source.display()
        )
    })
}

#[cfg(windows)]
fn replace(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_file(destination)
            .map_err(|error| format!("unable to replace {}: {error}", destination.display()))?;
    }
    fs::rename(source, destination).map_err(|error| {
        format!(
            "unable to replace {} with {}: {error}",
            destination.display(),
            source.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{CharacterRepository, write_atomic};

    #[test]
    fn default_collection_is_the_current_directory() {
        let repository = CharacterRepository::new(None);
        assert_eq!(repository.directory(), std::path::Path::new("."));
        assert_eq!(
            repository.resolve(std::path::Path::new("Legolas")),
            std::path::PathBuf::from("./Legolas.json")
        );
    }

    #[test]
    fn collection_resolution_and_atomic_replacement_are_stable() {
        let directory =
            std::env::temp_dir().join(format!("character-wizard-storage-{}", std::process::id()));
        let repository = CharacterRepository::new(Some(&directory));
        let path = repository.resolve(std::path::Path::new("Legolas"));
        assert_eq!(path, directory.join("Legolas.json"));
        write_atomic(&path, b"first").expect("first write");
        write_atomic(&path, b"second").expect("replacement write");
        assert_eq!(std::fs::read(&path).expect("read replacement"), b"second");
        std::fs::remove_file(path).expect("remove fixture");
        std::fs::remove_dir(directory).expect("remove directory");
    }
}
