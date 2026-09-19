use std::path::PathBuf;

use crate::i18n::{tr, tr_args};

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("help requested")]
    Help,
    #[error("version requested")]
    Version,
    #[error("expected a GitHub repository and pull request number")]
    InvalidArguments,
    #[error("repository must use owner/repo form, got {0:?}")]
    InvalidRepoSpec(String),
    #[error("invalid GitHub repository {kind}: {value:?}")]
    InvalidRepoSegment { kind: &'static str, value: String },
    #[error("pull request number must be a positive integer, got {0:?}")]
    InvalidPullRequest(String),
    #[error("expected a GitHub repository with --commit <hash>")]
    InvalidCommitArguments,
    #[error("expected a GitLab project and merge request number")]
    InvalidGitLabArguments,
    #[error("expected a GitLab project with --commit <hash>")]
    InvalidGitLabCommitArguments,
    #[error("GitLab project must use namespace/project form (subgroups allowed), got {0:?}")]
    InvalidGitLabProject(String),
    #[error("merge request number must be a positive integer, got {0:?}")]
    InvalidMergeRequest(String),
    #[error("commit hash must consist of 4 to 40 hexadecimal characters, got {0:?}")]
    InvalidCommitHash(String),
    #[error("--commit cannot be combined with --squash")]
    CommitWithSquash,
    #[error("missing value for {0}")]
    MissingOptionValue(String),
    #[error("unknown option {0}")]
    UnknownOption(String),
    #[error("failed to run curl: {0}")]
    DownloadCommand(#[source] std::io::Error),
    #[error("download failed with status {status:?}: {message}")]
    DownloadFailed {
        status: Option<i32>,
        message: String,
    },
    #[error("downloaded patch is not valid UTF-8: {0}")]
    PatchNotUtf8(#[from] std::string::FromUtf8Error),
    #[error("downloaded patch is empty")]
    EmptyPatch,
    #[error("downloaded content is not a Git diff")]
    InvalidDiff,
    #[error("failed to create output directory {path:?}: {source}")]
    CreateOutputDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write patch file {path:?}: {source}")]
    WritePatch {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl AppError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidArguments
            | Self::InvalidRepoSpec(_)
            | Self::InvalidRepoSegment { .. }
            | Self::InvalidPullRequest(_)
            | Self::InvalidCommitArguments
            | Self::InvalidGitLabArguments
            | Self::InvalidGitLabCommitArguments
            | Self::InvalidGitLabProject(_)
            | Self::InvalidMergeRequest(_)
            | Self::InvalidCommitHash(_)
            | Self::CommitWithSquash
            | Self::MissingOptionValue(_)
            | Self::UnknownOption(_) => 2,
            _ => 1,
        }
    }

    pub(crate) fn message(&self) -> String {
        match self {
            Self::Help | Self::Version => String::new(),
            Self::InvalidArguments => tr("expected a GitHub repository and pull request number"),
            Self::InvalidRepoSpec(value) => tr_args(
                "repository must use owner/repo form, got {value}",
                &[("value", quoted(value))],
            ),
            Self::InvalidRepoSegment { kind, value } => tr_args(
                "invalid GitHub repository {kind}: {value}",
                &[("kind", repo_segment_label(kind)), ("value", quoted(value))],
            ),
            Self::InvalidPullRequest(value) => tr_args(
                "pull request number must be a positive integer, got {value}",
                &[("value", quoted(value))],
            ),
            Self::InvalidCommitArguments => tr("expected a GitHub repository with --commit <hash>"),
            Self::InvalidGitLabArguments => {
                tr("expected a GitLab project and merge request number")
            }
            Self::InvalidGitLabCommitArguments => {
                tr("expected a GitLab project with --commit <hash>")
            }
            Self::InvalidGitLabProject(value) => tr_args(
                "GitLab project must use namespace/project form (subgroups allowed), got {value}",
                &[("value", quoted(value))],
            ),
            Self::InvalidMergeRequest(value) => tr_args(
                "merge request number must be a positive integer, got {value}",
                &[("value", quoted(value))],
            ),
            Self::InvalidCommitHash(value) => tr_args(
                "commit hash must consist of 4 to 40 hexadecimal characters, got {value}",
                &[("value", quoted(value))],
            ),
            Self::CommitWithSquash => tr("--commit cannot be combined with --squash"),
            Self::MissingOptionValue(option) => {
                tr_args("missing value for {option}", &[("option", option.clone())])
            }
            Self::UnknownOption(option) => {
                tr_args("unknown option {option}", &[("option", option.clone())])
            }
            Self::DownloadCommand(source) if source.kind() == std::io::ErrorKind::NotFound => {
                tr("curl was not found in PATH")
            }
            Self::DownloadCommand(source) => tr_args(
                "failed to run curl: {source}",
                &[("source", source.to_string())],
            ),
            Self::DownloadFailed { status, message } => match (status, message.is_empty()) {
                (Some(code), false) => tr_args(
                    "download failed with status {status}: {message}",
                    &[
                        ("status", code.to_string()),
                        ("message", message.to_string()),
                    ],
                ),
                (Some(code), true) => tr_args(
                    "download failed with status {status}",
                    &[("status", code.to_string())],
                ),
                (None, false) => tr_args(
                    "download failed: {message}",
                    &[("message", message.to_string())],
                ),
                (None, true) => tr("download failed"),
            },
            Self::PatchNotUtf8(source) => tr_args(
                "downloaded patch is not valid UTF-8: {source}",
                &[("source", source.to_string())],
            ),
            Self::EmptyPatch => tr("downloaded patch is empty"),
            Self::InvalidDiff => tr("downloaded content is not a Git diff"),
            Self::CreateOutputDir { path, source } => tr_args(
                "failed to create output directory {path}: {source}",
                &[
                    ("path", path.display().to_string()),
                    ("source", source.to_string()),
                ],
            ),
            Self::WritePatch { path, source }
                if source.kind() == std::io::ErrorKind::AlreadyExists =>
            {
                tr_args(
                    "refusing to overwrite existing patch file {path}; pass --force to replace it",
                    &[("path", path.display().to_string())],
                )
            }
            Self::WritePatch { path, source } => tr_args(
                "failed to write patch file {path}: {source}",
                &[
                    ("path", path.display().to_string()),
                    ("source", source.to_string()),
                ],
            ),
        }
    }
}

fn quoted(value: &str) -> String {
    format!("{value:?}")
}

fn repo_segment_label(kind: &str) -> String {
    match kind {
        "owner" => tr("owner"),
        "repo" => tr("repo"),
        other => other.to_string(),
    }
}
