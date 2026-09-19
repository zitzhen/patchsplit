//! Hand-rolled command line parsing (zero extra dependencies).
//!
//! [`Options`] holds global flags and grows as new flags are added;
//! [`Config`] pairs them with the resolved [`Host`], project and target.

use std::path::PathBuf;

use patchsplit::{split_patch_by_commit, PatchPart};

use crate::error::AppError;
use crate::host::{host_for, Host, Platform, Target};

/// Flag-style options shared by every platform and target.
#[derive(Debug)]
pub(crate) struct Options {
    pub output_dir: PathBuf,
    pub force: bool,
    pub squash: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("patches"),
            force: false,
            squash: false,
        }
    }
}

/// Fully resolved invocation: where to download from and how to write output.
#[derive(Debug)]
pub(crate) struct Config {
    pub platform: Platform,
    pub project: String,
    pub target: Target,
    pub options: Options,
    host: Box<dyn Host>,
}

impl Config {
    pub(crate) fn parse<I>(args: I) -> Result<Self, AppError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut options = Options::default();
        let mut gitlab = false;
        let mut commit = None;
        let mut positionals = Vec::new();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => return Err(AppError::Help),
                "-V" | "--version" => return Err(AppError::Version),
                "-f" | "--force" => options.force = true,
                "-s" | "--squash" => options.squash = true,
                "--gitlab" => gitlab = true,
                "-o" | "--out" => {
                    let option = arg.as_str().to_string();
                    let value = args.next().ok_or(AppError::MissingOptionValue(option))?;
                    options.output_dir = PathBuf::from(value);
                }
                value if value.starts_with("--out=") => {
                    options.output_dir = PathBuf::from(&value["--out=".len()..]);
                }
                "-commit" | "--commit" => {
                    let value = args
                        .next()
                        .ok_or(AppError::MissingOptionValue("--commit".to_string()))?;
                    commit = Some(value);
                }
                value if value.starts_with("--commit=") => {
                    commit = Some(value["--commit=".len()..].to_string());
                }
                value if value.starts_with("-commit=") => {
                    commit = Some(value["-commit=".len()..].to_string());
                }
                value if value.starts_with('-') => {
                    return Err(AppError::UnknownOption(value.to_string()));
                }
                value => positionals.push(value.to_string()),
            }
        }

        if commit.is_some() && options.squash {
            return Err(AppError::CommitWithSquash);
        }

        let platform = if gitlab {
            Platform::GitLab
        } else {
            Platform::GitHub
        };
        let host = host_for(platform);
        let (project, target) = host.resolve(&positionals, commit)?;

        Ok(Self {
            platform,
            project,
            target,
            options,
            host,
        })
    }

    pub(crate) fn patch_url(&self) -> String {
        match &self.target {
            Target::MergeRequest(number) => {
                let extension = if self.options.squash { "diff" } else { "patch" };
                self.host.merge_url(&self.project, *number, extension)
            }
            Target::Commit(hash) => self.host.commit_url(&self.project, hash),
        }
    }

    pub(crate) fn patch_parts(&self, patch: &str) -> Vec<PatchPart> {
        match &self.target {
            Target::Commit(hash) => {
                if patch.trim().is_empty() {
                    return Vec::new();
                }
                // A commit .patch is a single mail-formatted patch; keep it verbatim.
                vec![PatchPart {
                    index: 1,
                    commit: None,
                    subject: format!("commit {hash}"),
                    filename: format!("{hash}.patch"),
                    content: patch.to_string(),
                }]
            }
            Target::MergeRequest(number) => {
                if !self.options.squash {
                    return split_patch_by_commit(patch);
                }
                if patch.trim().is_empty() {
                    return Vec::new();
                }
                let (subject, filename) = self.host.squash_labels(*number);
                vec![PatchPart {
                    index: 1,
                    commit: None,
                    subject,
                    filename,
                    content: patch.to_string(),
                }]
            }
        }
    }
}
