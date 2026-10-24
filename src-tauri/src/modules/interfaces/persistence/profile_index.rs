use crate::modules::interfaces::document::DocumentFormat;
use crate::modules::interfaces::document::InterfaceDocument;
use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::document::UnavailableProfile;
use crate::modules::interfaces::persistence::profile_files::ProfileFiles;
use crate::modules::interfaces::validation::InterfaceValidator;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ProfileIndex {
    format_version: u32,
    revision: u64,
    profiles: Vec<String>,
}

impl ProfileIndex {
    const MAX_PROFILES: usize = 100;

    pub(super) fn load(
        directory: &Path,
    ) -> Result<(InterfaceWorkspace, Vec<UnavailableProfile>), String> {
        ProfileFiles::regular(directory, true)?;
        let index: Self = ProfileFiles::read(&directory.join("workspace.json"))?;

        if index.format_version != DocumentFormat::VERSION
            || index.profiles.len() > Self::MAX_PROFILES
        {
            return Err("Unsupported profile index or too many profiles".into());
        }

        let mut workspace = InterfaceWorkspace::empty();
        let mut unavailable = Vec::new();
        let mut names = HashSet::new();
        let mut validator = InterfaceValidator::new();

        workspace.revision = index.revision;

        let mut bytes = serde_json::to_vec(&workspace)
            .map_err(|error| error.to_string())?
            .len();

        for folder in index.profiles {
            ProfileFiles::component(&folder)?;

            if !names.insert(folder.to_lowercase()) {
                return Err("Duplicate profile folder in workspace index".into());
            }

            let path = directory.join(&folder);

            match Self::document(
                &path,
                &mut validator,
                &mut bytes,
                !workspace.documents.is_empty(),
            ) {
                Ok(document) => workspace.documents.push(document),
                Err(reason) => unavailable.push(UnavailableProfile {
                    fingerprint: ProfileFiles::fingerprint(&path)?,
                    folder,
                    reason,
                }),
            }
        }

        InterfaceValidator::new().validate(&workspace)?;

        Ok((workspace, unavailable))
    }

    fn document(
        path: &Path,
        validator: &mut InterfaceValidator,
        bytes: &mut usize,
        separator: bool,
    ) -> Result<InterfaceDocument, String> {
        let document = ProfileFiles::load(path)?;
        let size = serde_json::to_vec(&document)
            .map_err(|error| error.to_string())?
            .len();

        ProfileFiles::validate(&document)?;

        let size = size.saturating_add(usize::from(separator));

        if bytes.saturating_add(size) > ProfileFiles::MAX_BYTES {
            return Err("Profiles exceed the workspace size limit".into());
        }

        let mut next = validator.clone();

        next.add(&document)?;
        *validator = next;
        *bytes += size;

        Ok(document)
    }

    pub(super) fn save(
        directory: &Path,
        workspace: &InterfaceWorkspace,
        unavailable: &[UnavailableProfile],
        source: &Path,
    ) -> Result<(), String> {
        let kept = unavailable
            .iter()
            .filter(|profile| profile.fingerprint.is_some())
            .count();

        if workspace.documents.len().saturating_add(kept) > Self::MAX_PROFILES {
            return Err("Too many profiles, including unavailable profiles".into());
        }

        let mut profiles = Vec::new();
        let mut names = HashSet::from(["workspace.json".to_owned()]);

        names.extend(
            unavailable
                .iter()
                .map(|profile| profile.folder.to_lowercase()),
        );

        for document in &workspace.documents {
            let base = ProfileFiles::folder(&document.name);
            let mut folder = base.clone();
            let mut number = 1;

            while !names.insert(folder.to_lowercase()) {
                number += 1;
                folder = format!("{} ({number})", base.chars().take(56).collect::<String>());
            }

            ProfileFiles::save(&directory.join(&folder), document)?;
            profiles.push(folder);
        }

        for profile in unavailable
            .iter()
            .filter(|profile| profile.fingerprint.is_some())
        {
            ProfileFiles::copy(
                &source.join(&profile.folder),
                &directory.join(&profile.folder),
            )?;
            profiles.push(profile.folder.clone());
        }

        ProfileFiles::write(
            &directory.join("workspace.json"),
            &Self {
                format_version: workspace.format_version,
                revision: workspace.revision,
                profiles,
            },
        )
    }
}
