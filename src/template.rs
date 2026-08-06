//! Official character-sheet discovery, download, and cache handling.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TEMPLATE_NAME: &str = "character-sheet.pdf";
const TEMPLATE_DOWNLOAD_URL: &str =
    "https://media.dndbeyond.com/compendium-images/free-rules/ph/character-sheet.pdf";
const TEMPLATE_PAGE_URL: &str = "https://www.dndbeyond.com/resources/1779-d-d-character-sheets";
const MAX_TEMPLATE_BYTES: u64 = 20 * 1024 * 1024;

pub(crate) fn resolve_template(explicit: Option<&Path>) -> Result<PathBuf, String> {
    let cwd =
        env::current_dir().map_err(|error| format!("unable to read current directory: {error}"))?;
    let cache = cache_path()?;
    resolve_template_at(explicit, &cwd, &cache, &HttpDownloader)
}

trait Downloader {
    fn download(&self, destination: &Path) -> Result<(), String>;
}

struct HttpDownloader;

impl Downloader for HttpDownloader {
    fn download(&self, destination: &Path) -> Result<(), String> {
        let mut response = ureq::get(TEMPLATE_DOWNLOAD_URL)
            .call()
            .map_err(|error| format!("download request failed: {error}"))?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_TEMPLATE_BYTES)
            .read_to_vec()
            .map_err(|error| format!("unable to read downloaded template: {error}"))?;
        fs::write(destination, bytes)
            .map_err(|error| format!("unable to write {}: {error}", destination.display()))
    }
}

fn resolve_template_at(
    explicit: Option<&Path>,
    cwd: &Path,
    cache: &Path,
    downloader: &dyn Downloader,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        validate(path)?;
        println!("Using character sheet: {}", path.display());
        return Ok(path.to_owned());
    }

    let local = cwd.join(TEMPLATE_NAME);
    if local.exists() {
        if !local.is_file() {
            return Err(format!(
                "local character sheet is not a file: {}",
                local.display()
            ));
        }
        validate(&local).map_err(|error| {
            format!(
                "{error}\nThe local {} was not replaced. Supply a supported sheet with --template.",
                local.display()
            )
        })?;
        println!("Using local character sheet: {}", local.display());
        return Ok(local);
    }

    if cache.is_file() && validate(cache).is_ok() {
        println!("Using cached character sheet: {}", cache.display());
        return Ok(cache.to_owned());
    }

    println!("No supported character sheet was found.");
    println!("Downloading the official character sheet...");
    download_and_install(cache, downloader)?;
    println!("Template saved to {}", cache.display());
    Ok(cache.to_owned())
}

fn download_and_install(cache: &Path, downloader: &dyn Downloader) -> Result<(), String> {
    let parent = cache
        .parent()
        .ok_or_else(|| format!("cache path has no parent: {}", cache.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "unable to create cache directory {}: {error}",
            parent.display()
        )
    })?;
    let temporary = temporary_download_path(cache);

    let result = downloader
        .download(&temporary)
        .and_then(|()| validate(&temporary))
        .and_then(|()| install_download(&temporary, cache));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| {
        format!(
            "{error}\nDownload the supported sheet manually from {TEMPLATE_PAGE_URL} and pass it with --template."
        )
    })
}

fn install_download(temporary: &Path, cache: &Path) -> Result<(), String> {
    match fs::rename(temporary, cache) {
        Ok(()) => Ok(()),
        Err(error) if cache.exists() => {
            if validate(cache).is_ok() {
                fs::remove_file(temporary).map_err(|remove_error| {
                    format!(
                        "another process installed the cached template, but temporary file {} could not be removed: {remove_error}",
                        temporary.display()
                    )
                })?;
                return Ok(());
            }
            fs::remove_file(cache).map_err(|remove_error| {
                format!(
                    "unable to replace cached template {} after {error}: {remove_error}",
                    cache.display()
                )
            })?;
            fs::rename(temporary, cache)
                .map_err(|error| format!("unable to install {}: {error}", cache.display()))
        }
        Err(error) => Err(format!("unable to install {}: {error}", cache.display())),
    }
}

fn validate(path: &Path) -> Result<(), String> {
    crate::character_wizard_pdf_renderer::validate_template(path)
}

fn temporary_download_path(cache: &Path) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    cache.with_file_name(format!(
        ".{TEMPLATE_NAME}.download-{}-{nonce}",
        std::process::id()
    ))
}

fn cache_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("CHARACTER_WIZARD_CACHE_DIR") {
        return Ok(PathBuf::from(path).join(TEMPLATE_NAME));
    }
    platform_cache_root()
        .map(|root| root.join("character-wizard").join(TEMPLATE_NAME))
        .ok_or_else(|| {
            "unable to determine the user cache directory; supply --template PATH or set CHARACTER_WIZARD_CACHE_DIR"
                .to_owned()
        })
}

#[cfg(target_os = "windows")]
fn platform_cache_root() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn platform_cache_root() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Caches")))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_cache_root() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{Downloader, TEMPLATE_NAME, resolve_template_at};

    struct FixtureDownloader {
        source: PathBuf,
        calls: AtomicUsize,
    }

    impl Downloader for FixtureDownloader {
        fn download(&self, destination: &Path) -> Result<(), String> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            fs::copy(&self.source, destination)
                .map(|_| ())
                .map_err(|error| error.to_string())
        }
    }

    fn fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/character-sheet.pdf")
    }

    fn temporary_directory(label: &str) -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = env::temp_dir().join(format!(
            "character-wizard-template-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create temporary directory");
        path
    }

    #[test]
    fn explicit_template_has_first_priority() {
        let root = temporary_directory("explicit");
        let downloader = FixtureDownloader {
            source: fixture(),
            calls: AtomicUsize::new(0),
        };
        let resolved = resolve_template_at(
            Some(&fixture()),
            &root,
            &root.join("cache").join(TEMPLATE_NAME),
            &downloader,
        )
        .expect("explicit template");
        fs::remove_dir_all(root).expect("remove temporary directory");
        assert_eq!(resolved, fixture());
        assert_eq!(downloader.calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn local_template_precedes_the_cache() {
        let root = temporary_directory("local");
        let local = root.join(TEMPLATE_NAME);
        fs::copy(fixture(), &local).expect("copy local fixture");
        let downloader = FixtureDownloader {
            source: fixture(),
            calls: AtomicUsize::new(0),
        };
        let resolved = resolve_template_at(
            None,
            &root,
            &root.join("cache").join(TEMPLATE_NAME),
            &downloader,
        )
        .expect("local template");
        assert_eq!(resolved, local);
        assert_eq!(downloader.calls.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(root).expect("remove temporary directory");
    }

    #[test]
    fn valid_cached_template_is_reused_without_downloading() {
        let root = temporary_directory("cached");
        let cache = root.join("cache").join(TEMPLATE_NAME);
        fs::create_dir_all(cache.parent().expect("cache parent")).expect("create cache directory");
        fs::copy(fixture(), &cache).expect("copy cached fixture");
        let downloader = FixtureDownloader {
            source: fixture(),
            calls: AtomicUsize::new(0),
        };
        let resolved =
            resolve_template_at(None, &root, &cache, &downloader).expect("cached template");
        assert_eq!(resolved, cache);
        assert_eq!(downloader.calls.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(root).expect("remove temporary directory");
    }

    #[test]
    fn missing_template_is_downloaded_validated_and_cached() {
        let root = temporary_directory("download");
        let cache = root.join("cache").join(TEMPLATE_NAME);
        let downloader = FixtureDownloader {
            source: fixture(),
            calls: AtomicUsize::new(0),
        };
        let resolved =
            resolve_template_at(None, &root, &cache, &downloader).expect("downloaded template");
        assert_eq!(resolved, cache);
        assert_eq!(downloader.calls.load(Ordering::Relaxed), 1);
        crate::character_wizard_pdf_renderer::validate_template(&resolved)
            .expect("valid cached template");
        fs::remove_dir_all(root).expect("remove temporary directory");
    }

    #[test]
    fn invalid_local_template_is_never_replaced() {
        let root = temporary_directory("invalid-local");
        let local = root.join(TEMPLATE_NAME);
        fs::write(&local, b"not a PDF").expect("write invalid local file");
        let downloader = FixtureDownloader {
            source: fixture(),
            calls: AtomicUsize::new(0),
        };
        let error = resolve_template_at(
            None,
            &root,
            &root.join("cache").join(TEMPLATE_NAME),
            &downloader,
        )
        .expect_err("invalid local template");
        assert!(error.contains("was not replaced"));
        assert_eq!(fs::read(&local).expect("read local file"), b"not a PDF");
        assert_eq!(downloader.calls.load(Ordering::Relaxed), 0);
        fs::remove_dir_all(root).expect("remove temporary directory");
    }

    #[test]
    fn invalid_cached_template_is_replaced_by_a_valid_download() {
        let root = temporary_directory("invalid-cache");
        let cache = root.join("cache").join(TEMPLATE_NAME);
        fs::create_dir_all(cache.parent().expect("cache parent")).expect("create cache directory");
        fs::write(&cache, b"not a PDF").expect("write invalid cached file");
        let downloader = FixtureDownloader {
            source: fixture(),
            calls: AtomicUsize::new(0),
        };
        let resolved =
            resolve_template_at(None, &root, &cache, &downloader).expect("downloaded template");
        assert_eq!(resolved, cache);
        assert_eq!(downloader.calls.load(Ordering::Relaxed), 1);
        crate::character_wizard_pdf_renderer::validate_template(&resolved)
            .expect("valid cached template");
        fs::remove_dir_all(root).expect("remove temporary directory");
    }
}
