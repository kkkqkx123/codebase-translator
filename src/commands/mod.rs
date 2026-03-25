use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
};

pub trait Command {
    fn execute(&self, global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()>;

    fn get_project_path(&self) -> Option<&str> {
        None
    }
}

pub mod cache;
pub mod clean;
pub mod detect;
pub mod init;
pub mod translate;
pub mod validate;
pub mod verify;
