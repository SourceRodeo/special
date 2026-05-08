use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub(crate) struct TempProjectDir {
    path: PathBuf,
}

impl TempProjectDir {
    pub(crate) fn new(prefix: &str) -> Self {
        let path = unique_temp_project_path(prefix);
        std::fs::create_dir_all(&path).expect("temp project dir should be created");
        Self { path }
    }

    pub(crate) fn canonicalized(prefix: &str) -> Self {
        let path = unique_temp_project_path(prefix);
        std::fs::create_dir_all(&path).expect("temp project dir should be created");
        let path = path
            .canonicalize()
            .expect("temp project dir should canonicalize");
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

fn unique_temp_project_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()))
}

impl AsRef<Path> for TempProjectDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Deref for TempProjectDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Drop for TempProjectDir {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => eprintln!(
                "special tests: failed to remove temporary project directory {}: {error}",
                self.path.display()
            ),
        }
    }
}
