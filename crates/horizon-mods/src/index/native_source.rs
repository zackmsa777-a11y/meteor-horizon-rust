//! Versioned native-source `.hmod` packages. These are trusted native code, never content mods.
//! Reading and staging only validate/copy bytes; the launcher's existing trust gate owns compilation.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub const MANIFEST: &str = "horizon-native-source.json";
const LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub path: String,
    /// None means an addition; it must not already exist in the base.
    pub before: Option<String>,
    pub after: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub kind: String,
    pub id: String,
    pub version: String,
    pub game: String,
    pub horizon: String,
    pub features: Vec<String>,
    pub files: Vec<Change>,
}
pub struct Package {
    pub manifest: Manifest,
    pub files: BTreeMap<String, Vec<u8>>,
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(|part| {
            if part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' ']) {
                return false;
            }
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}
fn source_path(path: &str) -> bool {
    if !safe_path(path) {
        return false;
    }
    if matches!(path, "Cargo.toml" | "Cargo.lock") {
        return true;
    }
    let mut parts = path.split('/');
    if parts.next() != Some("crates") {
        return false;
    }
    let Some(krate) = parts.next() else {
        return false;
    };
    // Packages cannot replace the launcher, its trust controls, or the package loader.
    if !matches!(
        krate,
        "meteor-client"
            | "horizon-world"
            | "horizon-client"
            | "horizon-client-core"
            | "horizon-client-input"
            | "horizon-client-render"
    ) {
        return false;
    }
    path.ends_with(".rs") || path.ends_with(".toml")
}
impl Package {
    /// Dispatch hint only. A recognized package must still pass `read` before installation.
    pub fn recognizes(path: &Path) -> bool {
        if path.is_dir() {
            return path.join(MANIFEST).is_file();
        }
        fs::File::open(path)
            .ok()
            .and_then(|f| zip::ZipArchive::new(f).ok())
            .is_some_and(|mut z| z.by_name(MANIFEST).is_ok())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let files = if path.is_dir() {
            read_directory(path)?
        } else {
            read_zip(path)?
        };
        Self::from_files(files)
    }
    fn from_files(files: BTreeMap<String, Vec<u8>>) -> Result<Self, String> {
        let manifest: Manifest = serde_json::from_slice(
            files
                .get(MANIFEST)
                .ok_or("Missing native-source manifest")?,
        )
        .map_err(|e| format!("Native-source manifest: {e}"))?;
        if manifest.schema != 1
            || manifest.kind != "horizon-native-source"
            || !super::valid_id(&manifest.id.replace('-', "_"))
            || manifest.version.is_empty()
            || manifest.game.is_empty()
            || !super::native::revision(&manifest.horizon)
            || manifest.files.is_empty()
            || manifest.features.is_empty()
            || manifest.features.iter().any(|f| {
                f.is_empty()
                    || !f
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-/".contains(&b))
            })
        {
            return Err("Unsupported or invalid native-source manifest".into());
        }
        let mut expected = BTreeSet::from([MANIFEST.to_owned()]);
        let mut names = BTreeSet::new();
        for change in &manifest.files {
            if !source_path(&change.path)
                || !names.insert(change.path.to_ascii_lowercase())
                || !digest(&change.after)
                || change.before.as_ref().is_some_and(|s| !digest(s))
            {
                return Err(format!(
                    "Invalid or duplicate source change: {}",
                    change.path
                ));
            }
            let entry = format!("source/{}", change.path);
            let payload = files
                .get(&entry)
                .ok_or_else(|| format!("Missing source payload: {}", change.path))?;
            if std::str::from_utf8(payload).is_err() || payload.contains(&0) {
                return Err(format!("Native source must be UTF-8 text: {}", change.path));
            }
            if sha(payload) != change.after {
                return Err(format!("Source hash mismatch: {}", change.path));
            }
            expected.insert(entry);
        }
        for name in files.keys() {
            if !expected.contains(name)
                && !(name.starts_with("notices/") && safe_path(name) && name.ends_with(".txt"))
            {
                return Err(format!("Unexpected native-source entry: {name}"));
            }
        }
        if !files.keys().any(|n| n.starts_with("notices/")) {
            return Err("Native-source package needs license notices".into());
        }
        Ok(Self { manifest, files })
    }
    pub fn facts(&self) -> super::Facts {
        super::Facts {
            id: Some(self.manifest.id.clone()),
            version: Some(self.manifest.version.clone()),
            api: format!("native-source-1/{}", self.manifest.horizon),
            game: Some(self.manifest.game.clone()),
            ..Default::default()
        }
    }
    pub fn fingerprint(&self) -> String {
        let mut hash = Sha256::new();
        for (name, bytes) in &self.files {
            hash.update(name.as_bytes());
            hash.update([0]);
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        format!("{:x}", hash.finalize())
    }
    /// The destination must be a fresh directory. Verify every base hash before any writes.
    pub fn stage(&self, base: &Path, destination: &Path) -> Result<(), String> {
        let base = base.canonicalize().map_err(|e| e.to_string())?;
        let parent = destination
            .parent()
            .ok_or("Staging directory has no parent")?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let name = destination
            .file_name()
            .ok_or("Staging directory has no name")?;
        let destination = parent.join(name);
        if destination.starts_with(&base) || base.starts_with(&destination) || destination.exists()
        {
            return Err("Native-source staging must be a new directory outside the base".into());
        }
        let revision = fs::read_to_string(base.join(".horizon-revision"))
            .map_err(|e| format!("Base revision: {e}"))?;
        if revision.trim() != self.manifest.horizon {
            return Err("Native-source base revision mismatch".into());
        }
        // The cache may have changed since download. Validate affected files and refuse links.
        for change in &self.manifest.files {
            let path = beneath_without_links(&base, &change.path)?;
            match (&change.before, fs::read(&path)) {
                (Some(expected), Ok(bytes)) if sha(&bytes) == *expected => {}
                (None, Err(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
                _ => return Err(format!("Native-source base file mismatch: {}", change.path)),
            }
        }
        copy_tree(&base, &destination)?;
        for change in &self.manifest.files {
            let path = destination.join(&change.path);
            fs::create_dir_all(path.parent().ok_or("Source path has no parent")?)
                .map_err(|e| e.to_string())?;
            fs::write(&path, &self.files[&format!("source/{}", change.path)])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
fn beneath_without_links(base: &Path, relative: &str) -> Result<PathBuf, String> {
    let mut path = base.to_owned();
    for part in relative.split('/') {
        path.push(part);
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && metadata.file_type().is_symlink()
        {
            return Err(format!("Source path is a link: {relative}"));
        }
    }
    Ok(path)
}
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err("Source tree contains a link".into());
        }
        if kind.is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        } else if kind.is_file() {
            fs::copy(entry.path(), to.join(entry.file_name())).map_err(|e| e.to_string())?;
        } else {
            return Err("Source tree contains a non-file entry".into());
        }
    }
    Ok(())
}
fn read_zip(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if zip.len() > 2048 {
        return Err("Too many native-source entries".into());
    }
    let mut files = BTreeMap::new();
    let mut names = BTreeSet::new();
    let mut total = 0;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_owned();
        if !safe_path(&name)
            || !names.insert(name.to_ascii_lowercase())
            || entry
                .unix_mode()
                .is_some_and(|m| m & 0o170000 != 0 && m & 0o170000 != 0o100000)
        {
            return Err(format!("Unsafe or duplicate native-source path: {name}"));
        }
        let mut bytes = Vec::new();
        (&mut entry)
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        total += bytes.len() as u64;
        if total > LIMIT {
            return Err("Native-source package exceeds 64 MiB".into());
        }
        files.insert(name, bytes);
    }
    Ok(files)
}
fn read_directory(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    fn walk(
        root: &Path,
        at: &Path,
        files: &mut BTreeMap<String, Vec<u8>>,
        names: &mut BTreeSet<String>,
        total: &mut u64,
    ) -> Result<(), String> {
        for entry in fs::read_dir(at).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() {
                return Err("Native-source package contains a link".into());
            }
            if kind.is_dir() {
                walk(root, &entry.path(), files, names, total)?;
            } else if kind.is_file() {
                let path = entry.path();
                let name = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                if !safe_path(&name) || !names.insert(name.to_ascii_lowercase()) {
                    return Err("Unsafe or duplicate source path".into());
                }
                let mut bytes = Vec::new();
                fs::File::open(path)
                    .map_err(|e| e.to_string())?
                    .take(LIMIT + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                *total += bytes.len() as u64;
                if *total > LIMIT || files.len() >= 2048 {
                    return Err("Native-source package exceeds limits".into());
                }
                files.insert(name, bytes);
            } else {
                return Err("Native-source package contains a non-file entry".into());
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files, &mut BTreeSet::new(), &mut 0)?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "hmod-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn files() -> BTreeMap<String, Vec<u8>> {
        let manifest = Manifest {
            schema: 1,
            kind: "horizon-native-source".into(),
            id: "meteor-client".into(),
            version: "0.1.0".into(),
            game: "26.3".into(),
            horizon: "a".repeat(40),
            features: vec!["horizon-client/meteor".into()],
            files: vec![
                Change {
                    path: "Cargo.toml".into(),
                    before: Some(sha(b"base")),
                    after: sha(b"changed"),
                },
                Change {
                    path: "crates/meteor-client/src/lib.rs".into(),
                    before: None,
                    after: sha(b"// new source"),
                },
            ],
        };
        BTreeMap::from([
            (MANIFEST.into(), serde_json::to_vec(&manifest).unwrap()),
            ("source/Cargo.toml".into(), b"changed".to_vec()),
            (
                "source/crates/meteor-client/src/lib.rs".into(),
                b"// new source".to_vec(),
            ),
            ("notices/license.txt".into(), b"test fixture".to_vec()),
        ])
    }
    fn archive(path: &Path, entries: BTreeMap<String, Vec<u8>>) {
        let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
        for (name, bytes) in entries {
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.finish().unwrap();
    }
    fn base(at: &Path) -> PathBuf {
        let base = at.join("base");
        fs::create_dir(&base).unwrap();
        fs::write(base.join(".horizon-revision"), "a".repeat(40)).unwrap();
        fs::write(base.join("Cargo.toml"), "base").unwrap();
        base
    }
    #[test]
    fn valid_archive_stages_without_modifying_base_and_is_not_content() {
        let dir = Scratch::new();
        let path = dir.0.join("meteor.hmod");
        archive(&path, files());
        assert!(Package::recognizes(&path));
        assert!(crate::Package::read(&path).is_err());
        let package = Package::read(&path).unwrap();
        let base = base(&dir.0);
        let stage = dir.0.join("stage");
        package.stage(&base, &stage).unwrap();
        assert_eq!(fs::read(base.join("Cargo.toml")).unwrap(), b"base");
        assert_eq!(fs::read(stage.join("Cargo.toml")).unwrap(), b"changed");
        assert_eq!(
            fs::read(stage.join("crates/meteor-client/src/lib.rs")).unwrap(),
            b"// new source"
        );
        assert!(package.stage(&base, &stage).is_err());
    }
    #[test]
    fn rejects_wrong_revision_and_modified_base_before_staging() {
        let dir = Scratch::new();
        let base = base(&dir.0);
        let stage = dir.0.join("stage");
        let package = Package::from_files(files()).unwrap();
        fs::write(base.join(".horizon-revision"), "b".repeat(40)).unwrap();
        assert!(
            package
                .stage(&base, &stage)
                .unwrap_err()
                .contains("revision mismatch")
        );
        assert!(!stage.exists());
        fs::write(base.join(".horizon-revision"), "a".repeat(40)).unwrap();
        fs::write(base.join("Cargo.toml"), "tampered").unwrap();
        assert!(
            package
                .stage(&base, &stage)
                .unwrap_err()
                .contains("base file mismatch")
        );
        assert!(!stage.exists());
    }
    #[test]
    fn rejects_tampered_payload_and_unlisted_entries() {
        let mut changed = files();
        changed.insert("source/Cargo.toml".into(), b"tampered".to_vec());
        assert!(
            Package::from_files(changed)
                .err()
                .unwrap()
                .contains("hash mismatch")
        );
        let mut changed = files();
        changed.insert("source/launcher/src/main.rs".into(), b"unexpected".to_vec());
        assert!(Package::from_files(changed).is_err());
    }
    #[test]
    fn rejects_archive_traversal_and_case_aliases() {
        let dir = Scratch::new();
        let path = dir.0.join("bad.hmod");
        for name in [
            "../outside",
            "source/../outside",
            "source/CARGO.toml",
            "notices/CON.txt",
            "notices/a:stream.txt",
        ] {
            let mut changed = files();
            changed.insert(name.into(), Vec::new());
            archive(&path, changed);
            assert!(Package::read(&path).is_err(), "accepted {name}");
        }
    }
    #[test]
    fn source_allowlist_excludes_loader_and_launcher() {
        for path in [
            "launcher/src/main.rs",
            "crates/horizon-mods/src/index.rs",
            "../Cargo.toml",
            "crates/meteor-client/a.exe",
            "crates/meteor-client/CON.rs",
        ] {
            assert!(!source_path(path));
        }
        assert!(source_path("crates/meteor-client/src/lib.rs"));
    }
}
