//! Platform-specific behavior hidden behind the [`Host`] trait.
//!
//! Adding a platform (or a self-hosted variant of an existing one) means
//! providing a new `impl Host`; the CLI flow in `main` stays untouched.

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Platform {
    GitHub,
    GitLab,
}

#[derive(Debug)]
pub(crate) enum Target {
    MergeRequest(u64),
    Commit(String),
}

/// Everything that differs between hosting platforms.
pub(crate) trait Host: std::fmt::Debug {
    /// Turn positional arguments and the optional `--commit` value into a
    /// validated `(project, target)` pair.
    fn resolve(
        &self,
        positionals: &[String],
        commit: Option<String>,
    ) -> Result<(String, Target), AppError>;

    /// URL of a merge request / pull request patch stream.
    /// `extension` is `patch` (per-commit mbox) or `diff` (squashed net diff).
    fn merge_url(&self, project: &str, number: u64, extension: &str) -> String;

    /// URL of a single commit's `.patch`.
    fn commit_url(&self, project: &str, hash: &str) -> String;

    /// `(subject, filename)` used when a merge request is written squashed.
    fn squash_labels(&self, number: u64) -> (String, String);
}

pub(crate) fn host_for(platform: Platform) -> Box<dyn Host> {
    match platform {
        Platform::GitHub => Box::new(GitHub),
        Platform::GitLab => Box::new(GitLab),
    }
}

#[derive(Debug)]
pub(crate) struct GitHub;

impl Host for GitHub {
    fn resolve(
        &self,
        positionals: &[String],
        commit: Option<String>,
    ) -> Result<(String, Target), AppError> {
        match commit {
            Some(hash) => {
                let (owner, repo) = match positionals {
                    [repo_spec] => parse_repo_spec(repo_spec)?,
                    [owner, repo] => {
                        validate_repo_segment("owner", owner)?;
                        validate_repo_segment("repo", repo)?;
                        (owner.clone(), repo.clone())
                    }
                    _ => return Err(AppError::InvalidCommitArguments),
                };
                validate_commit_hash(&hash)?;
                Ok((format!("{owner}/{repo}"), Target::Commit(hash)))
            }
            None => {
                let (owner, repo, pull_request) = match positionals {
                    [repo_spec, pull_request] => {
                        let (owner, repo) = parse_repo_spec(repo_spec)?;
                        (owner, repo, parse_pull_request(pull_request)?)
                    }
                    [owner, repo, pull_request] => {
                        validate_repo_segment("owner", owner)?;
                        validate_repo_segment("repo", repo)?;
                        (
                            owner.clone(),
                            repo.clone(),
                            parse_pull_request(pull_request)?,
                        )
                    }
                    _ => return Err(AppError::InvalidArguments),
                };
                Ok((
                    format!("{owner}/{repo}"),
                    Target::MergeRequest(pull_request),
                ))
            }
        }
    }

    fn merge_url(&self, project: &str, number: u64, extension: &str) -> String {
        // GitHub's PR diff represents the net change; .patch contains each commit.
        format!("https://github.com/{project}/pull/{number}.{extension}")
    }

    fn commit_url(&self, project: &str, hash: &str) -> String {
        format!("https://github.com/{project}/commit/{hash}.patch")
    }

    fn squash_labels(&self, number: u64) -> (String, String) {
        (format!("PR #{number}"), format!("pr-{number}.patch"))
    }
}

#[derive(Debug)]
pub(crate) struct GitLab;

impl Host for GitLab {
    fn resolve(
        &self,
        positionals: &[String],
        commit: Option<String>,
    ) -> Result<(String, Target), AppError> {
        // GitLab project paths may contain subgroups, e.g. group/subgroup/project.
        match commit {
            Some(hash) => {
                if positionals.is_empty() {
                    return Err(AppError::InvalidGitLabCommitArguments);
                }
                let project = positionals.join("/");
                validate_gitlab_project(&project)?;
                validate_commit_hash(&hash)?;
                Ok((project, Target::Commit(hash)))
            }
            None => {
                let Some((merge_request, project_parts)) = positionals.split_last() else {
                    return Err(AppError::InvalidGitLabArguments);
                };
                if project_parts.is_empty() {
                    return Err(AppError::InvalidGitLabArguments);
                }
                let project = project_parts.join("/");
                validate_gitlab_project(&project)?;
                let merge_request = parse_merge_request(merge_request)?;
                Ok((project, Target::MergeRequest(merge_request)))
            }
        }
    }

    fn merge_url(&self, project: &str, number: u64, extension: &str) -> String {
        format!("https://gitlab.com/{project}/-/merge_requests/{number}.{extension}")
    }

    fn commit_url(&self, project: &str, hash: &str) -> String {
        format!("https://gitlab.com/{project}/-/commit/{hash}.patch")
    }

    fn squash_labels(&self, number: u64) -> (String, String) {
        (format!("MR #{number}"), format!("mr-{number}.patch"))
    }
}

fn parse_repo_spec(value: &str) -> Result<(String, String), AppError> {
    let Some((owner, repo)) = value.split_once('/') else {
        return Err(AppError::InvalidRepoSpec(value.to_string()));
    };

    if repo.contains('/') {
        return Err(AppError::InvalidRepoSpec(value.to_string()));
    }

    validate_repo_segment("owner", owner)?;
    validate_repo_segment("repo", repo)?;
    Ok((owner.to_string(), repo.to_string()))
}

fn validate_repo_segment(kind: &'static str, value: &str) -> Result<(), AppError> {
    if value.is_empty()
        || value.chars().any(|character| {
            character == '/' || character.is_whitespace() || character.is_control()
        })
    {
        return Err(AppError::InvalidRepoSegment {
            kind,
            value: value.to_string(),
        });
    }

    Ok(())
}

fn parse_pull_request(value: &str) -> Result<u64, AppError> {
    let pull_request = value
        .parse::<u64>()
        .map_err(|_| AppError::InvalidPullRequest(value.to_string()))?;

    if pull_request == 0 {
        Err(AppError::InvalidPullRequest(value.to_string()))
    } else {
        Ok(pull_request)
    }
}

fn parse_merge_request(value: &str) -> Result<u64, AppError> {
    let merge_request = value
        .parse::<u64>()
        .map_err(|_| AppError::InvalidMergeRequest(value.to_string()))?;

    if merge_request == 0 {
        Err(AppError::InvalidMergeRequest(value.to_string()))
    } else {
        Ok(merge_request)
    }
}

fn validate_gitlab_project(value: &str) -> Result<(), AppError> {
    // GitLab projects live under a namespace and may be nested in subgroups.
    let segments: Vec<&str> = value.split('/').collect();
    let valid = segments.len() >= 2
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && !segment
                    .chars()
                    .any(|character| character.is_whitespace() || character.is_control())
        });

    if valid {
        Ok(())
    } else {
        Err(AppError::InvalidGitLabProject(value.to_string()))
    }
}

fn validate_commit_hash(value: &str) -> Result<(), AppError> {
    // Git abbreviations are at least 4 hex characters; full SHAs are 40.
    let valid =
        (4..=40).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_hexdigit());

    if valid {
        Ok(())
    } else {
        Err(AppError::InvalidCommitHash(value.to_string()))
    }
}
