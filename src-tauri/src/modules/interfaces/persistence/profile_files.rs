use crate::modules::interfaces::document::DocumentFormat;
use crate::modules::interfaces::document::InterfaceDocument;
use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::validation::InterfaceValidator;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub(crate) struct ProfileFiles;

impl ProfileFiles {
    pub(crate) const MAX_BYTES: usize = 32 * 1024 * 1024;
    const MAX_DEPTH: usize = 8;

    pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
        Self::regular(path, false)?;
        let file = fs::File::open(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        let mut bytes = Vec::new();

        file.take(Self::MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;

        if bytes.len() > Self::MAX_BYTES {
            return Err(format!("Profile file is too large: {}", path.display()));
        }

        serde_json::from_slice(&bytes).map_err(|error| {
            if DocumentFormat::unsupported(&error) {
                format!(
                    "Profile {} uses unsupported fields or values; a newer Aurion may be required: {error}",
                    path.display()
                )
            } else {
                format!("Invalid profile file {}: {error}", path.display())
            }
        })
    }

    pub(crate) fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;

        if bytes.len() > Self::MAX_BYTES {
            return Err("Profile file is too large".into());
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("Could not create {}: {error}", path.display()))?;

        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())
    }

    pub(crate) fn load(directory: &Path) -> Result<InterfaceDocument, String> {
        Self::regular(directory, true)?;
        let mut document: InterfaceDocument = Self::read(&directory.join("interface.json"))?;
        let locales = directory.join("locales");

        if !locales.try_exists().map_err(|error| error.to_string())? {
            return Ok(document);
        }

        Self::regular(&locales, true)?;

        for entry in fs::read_dir(&locales).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();

            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let locale = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or("Invalid profile locale filename")?
                .to_owned();
            let texts: BTreeMap<String, String> = Self::read(&path)?;

            if document
                .localization
                .translations
                .insert(locale, texts)
                .is_some()
            {
                return Err("A profile language exists both inline and in locales".into());
            }

            Self::validate(&document)?;
        }

        Ok(document)
    }

    pub(crate) fn save(directory: &Path, document: &InterfaceDocument) -> Result<(), String> {
        Self::validate(document)?;
        fs::create_dir(directory).map_err(|error| error.to_string())?;
        let mut definition = document.clone();
        let translations = std::mem::take(&mut definition.localization.translations);

        Self::write(&directory.join("interface.json"), &definition)?;
        let locales = directory.join("locales");

        fs::create_dir(&locales).map_err(|error| error.to_string())?;

        for (locale, texts) in translations {
            Self::write(&locales.join(format!("{locale}.json")), &texts)?;
        }

        Ok(())
    }

    pub(crate) fn validate(document: &InterfaceDocument) -> Result<(), String> {
        let mut workspace = InterfaceWorkspace::empty();

        workspace.documents.push(document.clone());
        InterfaceValidator::new().validate(&workspace)?;
        Self::limit(&workspace)
    }

    pub(crate) fn limit(workspace: &InterfaceWorkspace) -> Result<(), String> {
        if serde_json::to_vec(workspace)
            .map_err(|error| error.to_string())?
            .len()
            > Self::MAX_BYTES
        {
            return Err("Profiles exceed the workspace size limit".into());
        }

        Ok(())
    }

    pub(crate) fn fingerprint(directory: &Path) -> Result<Option<String>, String> {
        if !directory.try_exists().map_err(|error| error.to_string())? {
            return Ok(None);
        }

        let mut hash = Sha256::new();

        for (relative, bytes) in Self::tree(directory)? {
            hash.update(relative.to_string_lossy().as_bytes());
            hash.update([0]);
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(&bytes);
        }

        Ok(Some(format!("{:x}", hash.finalize())))
    }

    pub(crate) fn copy(source: &Path, target: &Path) -> Result<(), String> {
        let files = Self::tree(source)?;

        fs::create_dir(target).map_err(|error| error.to_string())?;

        for (relative, bytes) in files {
            let path = target.join(&relative);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|error| format!("Could not create {}: {error}", path.display()))?;

            file.write_all(&bytes).map_err(|error| error.to_string())?;
            file.sync_all().map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    fn tree(directory: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
        let mut files = Vec::new();
        let mut pending = vec![(PathBuf::new(), 0)];
        let mut total = 0;

        Self::regular(directory, true)?;

        while let Some((relative, depth)) = pending.pop() {
            if depth > Self::MAX_DEPTH {
                return Err(format!(
                    "Profile folder is too deep: {}",
                    directory.display()
                ));
            }

            for entry in
                fs::read_dir(directory.join(&relative)).map_err(|error| error.to_string())?
            {
                let name = relative.join(entry.map_err(|error| error.to_string())?.file_name());
                let path = directory.join(&name);
                let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;

                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    pending.push((name, depth + 1));

                    continue;
                }

                Self::regular(&path, false)?;
                let mut bytes = Vec::new();

                fs::File::open(&path)
                    .and_then(|file| {
                        file.take(Self::MAX_BYTES as u64 + 1)
                            .read_to_end(&mut bytes)
                    })
                    .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
                total += bytes.len();

                if total > Self::MAX_BYTES {
                    return Err(format!(
                        "Profile folder is too large: {}",
                        directory.display()
                    ));
                }

                files.push((name, bytes));
            }
        }

        files.sort_by(|left, right| left.0.cmp(&right.0));

        Ok(files)
    }

    pub(crate) fn regular(path: &Path, directory: bool) -> Result<(), String> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?;

        if metadata.file_type().is_symlink()
            || (directory && !metadata.is_dir())
            || (!directory && !metadata.is_file())
        {
            return Err(format!(
                "Expected a regular profile path: {}",
                path.display()
            ));
        }

        Ok(())
    }

    pub(crate) fn folder(name: &str) -> String {
        let safe: String = name
            .chars()
            .take(63)
            .map(|character| {
                if character.is_control() || "<>:\"/\\|?*".contains(character) {
                    '_'
                } else {
                    character
                }
            })
            .collect();
        let safe = safe.trim_matches([' ', '.']);
        let stem = safe
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();

        if safe.is_empty()
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || stem.starts_with("COM")
            || stem.starts_with("LPT")
        {
            return format!("_{}", safe.chars().take(62).collect::<String>());
        }

        safe.to_owned()
    }

    pub(crate) fn component(value: &str) -> Result<(), String> {
        if value.is_empty() || Self::folder(value) != value {
            return Err("Invalid profile folder name".into());
        }

        Ok(())
    }
}
