use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::document::UnavailableProfile;
use crate::modules::interfaces::persistence::profile_directory::ProfileDirectory;
use crate::modules::interfaces::persistence::profile_files::ProfileFiles;
use crate::modules::interfaces::persistence::profile_index::ProfileIndex;
use crate::modules::interfaces::validation::InterfaceValidator;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

pub(crate) type StoredProfiles = (InterfaceWorkspace, Vec<UnavailableProfile>);

pub(crate) struct InterfaceRepository {
    profiles: PathBuf,
}

impl InterfaceRepository {
    pub(crate) fn new(profiles: PathBuf) -> Self {
        Self { profiles }
    }

    pub(crate) fn load(&self) -> Result<StoredProfiles, String> {
        let _lock = self.lock()?;

        self.read()
    }

    fn lock(&self) -> Result<File, String> {
        ProfileDirectory::new(self.profiles.clone()).lock()
    }

    fn read(&self) -> Result<StoredProfiles, String> {
        ProfileDirectory::new(self.profiles.clone()).recover()?;

        if !self
            .profiles
            .try_exists()
            .map_err(|error| error.to_string())?
        {
            return Ok((InterfaceWorkspace::empty(), Vec::new()));
        }

        ProfileIndex::load(&self.profiles)
    }

    pub(crate) fn save(
        &self,
        workspace: &InterfaceWorkspace,
        keep: &[UnavailableProfile],
        expected: &StoredProfiles,
    ) -> Result<StoredProfiles, String> {
        let _lock = self.lock()?;

        if !Self::unchanged(&self.read()?, expected)? {
            return Err(
                "Save failed: the file changed externally. Your edits remain in the open editor."
                    .into(),
            );
        }

        InterfaceValidator::new().validate(workspace)?;
        ProfileFiles::limit(workspace)?;

        self.save_profiles(&self.profiles, workspace, keep, expected)
    }

    fn unchanged(current: &StoredProfiles, expected: &StoredProfiles) -> Result<bool, String> {
        Ok(current.1.len() == expected.1.len()
            && current
                .1
                .iter()
                .zip(&expected.1)
                .all(|(left, right)| left.same(right))
            && serde_json::to_value(&current.0).map_err(|error| error.to_string())?
                == serde_json::to_value(&expected.0).map_err(|error| error.to_string())?)
    }

    fn save_profiles(
        &self,
        profiles: &Path,
        workspace: &InterfaceWorkspace,
        keep: &[UnavailableProfile],
        expected: &StoredProfiles,
    ) -> Result<StoredProfiles, String> {
        let mut directory = ProfileDirectory::new(profiles.to_path_buf());
        let stage = directory.stage()?;
        let result = ProfileIndex::save(&stage, workspace, keep, profiles).and_then(|()| {
            let stored = ProfileIndex::load(&stage)?;

            if !Self::unchanged(&self.read()?, expected)? {
                return Err(
                    "Profiles changed externally while saving. Your edits remain in the editor."
                        .into(),
                );
            }

            directory.commit(&stage)?;

            Ok(stored)
        });

        if result.is_err() && stage.exists() {
            let _ = directory.discard(&stage);
        }

        result
    }
}
