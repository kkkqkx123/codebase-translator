use crate::{
    config::{global::GlobalConfig, project::ProjectConfig},
    core::error::Result,
};

pub trait Command {
    fn execute(&self, global_config: &GlobalConfig, project_config: &ProjectConfig) -> Result<()>;
}

pub mod cache;
pub mod init;
pub mod translate;
pub mod validate;
pub mod verify;
