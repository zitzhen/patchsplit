use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

mod config;
mod download;
mod error;
mod host;
mod i18n;

use config::Config;
use download::{CurlDownloader, Downloader};
use error::AppError;
use i18n::{tr, tr_args};
use patchsplit::PatchPart;

fn main() {
    i18n::init();

    match run() {
        Ok(()) => {}
        Err(AppError::Help) => {
            println!("{}", usage());
        }
        Err(AppError::Version) => {
            println!(
                "{}",
                tr_args(
                    "patchsplit {version}",
                    &[("version", patchsplit::version().to_string())]
                )
            );
        }
        Err(error) => {
            eprintln!(
                "{}",
                tr_args("error: {message}", &[("message", error.message())])
            );
            eprintln!();
            eprintln!("{}", usage());
            std::process::exit(error.exit_code());
        }
    }
}

fn run() -> Result<(), AppError> {
    let config = Config::parse(env::args().skip(1))?;
    let url = config.patch_url();
    let downloader = CurlDownloader;
    let patch = downloader.fetch(&url)?;
    if config.options.squash && !patch.trim().is_empty() && !patch.starts_with("diff --git ") {
        return Err(AppError::InvalidDiff);
    }
    let parts = config.patch_parts(&patch);

    if parts.is_empty() {
        return Err(AppError::EmptyPatch);
    }

    let written = write_parts(&parts, &config.options.output_dir, config.options.force)?;

    println!("{}", tr_args("downloaded {url}", &[("url", url)]));
    println!(
        "{}",
        tr_args(
            "wrote {count} patch file(s) to {directory}",
            &[
                ("count", written.len().to_string()),
                ("directory", config.options.output_dir.display().to_string()),
            ]
        )
    );
    for path in written {
        println!("{}", path.display());
    }

    Ok(())
}

fn write_parts(
    parts: &[PatchPart],
    output_dir: &Path,
    force: bool,
) -> Result<Vec<PathBuf>, AppError> {
    fs::create_dir_all(output_dir).map_err(|source| AppError::CreateOutputDir {
        path: output_dir.to_path_buf(),
        source,
    })?;

    let mut written = Vec::with_capacity(parts.len());
    for part in parts {
        let path = output_dir.join(&part.filename);
        let mut options = OpenOptions::new();
        options.write(true);

        // Refuse overwrites by default so reruns do not replace manually edited patches.
        if force {
            options.create(true).truncate(true);
        } else {
            options.create_new(true);
        }

        let mut file = options.open(&path).map_err(|source| AppError::WritePatch {
            path: path.clone(),
            source,
        })?;
        file.write_all(part.content.as_bytes())
            .map_err(|source| AppError::WritePatch {
                path: path.clone(),
                source,
            })?;
        written.push(path);
    }

    Ok(written)
}

fn usage() -> String {
    tr("Usage:\n  patchsplit <owner/repo> <pr-number> [--out <dir>] [--force] [--squash]\n  patchsplit <owner> <repo> <pr-number> [--out <dir>] [--force] [--squash]\n  patchsplit <owner/repo> --commit <hash> [--out <dir>] [--force]\n  patchsplit <owner> <repo> --commit <hash> [--out <dir>] [--force]\n  patchsplit --gitlab <namespace/project> <mr-number> [--out <dir>] [--force] [--squash]\n  patchsplit --gitlab <namespace/project> --commit <hash> [--out <dir>] [--force]\n\nOptions:\n  -o, --out <dir>   Output directory for patch files [default: patches]\n  -f, --force       Overwrite existing patch files\n  -s, --squash      Write the net diff as one patch instead of splitting by commit\n      --gitlab      Download from gitlab.com (merge requests and commits)\n      --commit <hash> Download one commit's .patch (short or full hash)\n  -h, --help        Show this help\n  -V, --version     Show version\n\nExamples:\n  patchsplit rust-lang/rust 12345\n  patchsplit openai codex 42 -o pr-42-patches\n  patchsplit openai/codex 42 --squash\n  patchsplit zitzhen patchsplit -commit b430113\n  patchsplit --gitlab zitzhen/patchsplit 1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Platform;

    fn config(args: &[&str]) -> Config {
        Config::parse(args.iter().map(|arg| arg.to_string())).unwrap()
    }

    #[test]
    fn default_still_downloads_per_commit_patches() {
        let config = config(&["owner/repo", "42"]);
        assert!(!config.options.squash);
        assert_eq!(
            config.patch_url(),
            "https://github.com/owner/repo/pull/42.patch"
        );
        let patch = format!(
            "From {} Mon Sep 17 00:00:00 2001\nSubject: [PATCH 1/2] First\n\nfirst\nFrom {} Mon Sep 17 00:00:00 2001\nSubject: [PATCH 2/2] Second\n\nsecond\n",
            "1".repeat(40), "2".repeat(40)
        );
        assert_eq!(config.patch_parts(&patch).len(), 2);
    }

    #[test]
    fn squash_uses_net_diff_for_both_argument_forms() {
        for args in [
            vec!["owner/repo", "42", "--squash", "--out=combined", "--force"],
            vec!["-s", "owner", "repo", "42", "-o", "combined", "-f"],
        ] {
            let config = config(&args);
            assert_eq!(
                config.patch_url(),
                "https://github.com/owner/repo/pull/42.diff"
            );
            assert_eq!(config.options.output_dir, PathBuf::from("combined"));
            assert!(config.options.force);
            let diff =
                "diff --git a/file b/file\n--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+final\n";
            let parts = config.patch_parts(diff);
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].filename, "pr-42.patch");
            assert_eq!(parts[0].content, diff);
            assert!(config.patch_parts(" \n\t").is_empty());
        }
    }

    #[test]
    fn commit_mode_downloads_a_single_commit_patch() {
        const FULL_HASH: &str = "b4301133226e5c3a464cff9649de0b321c0b0a2e";
        let full_double_dash = format!("--commit={FULL_HASH}");
        let full_single_dash = format!("-commit={FULL_HASH}");
        let cases: Vec<(Vec<&str>, &str)> = vec![
            (vec!["zitzhen/patchsplit", "-commit", "b430113"], "b430113"),
            (
                vec!["zitzhen", "patchsplit", "--commit", "b430113"],
                "b430113",
            ),
            (vec!["zitzhen/patchsplit", &full_double_dash], FULL_HASH),
            (vec!["zitzhen/patchsplit", &full_single_dash], FULL_HASH),
        ];
        let patch = format!(
            "From {FULL_HASH} Mon Sep 17 00:00:00 2001\nSubject: [PATCH] One\n\ndiff --git a/a b/a\n"
        );

        for (args, hash) in cases {
            let config = config(&args);
            assert_eq!(
                config.patch_url(),
                format!("https://github.com/zitzhen/patchsplit/commit/{hash}.patch")
            );
            let parts = config.patch_parts(&patch);
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].filename, format!("{hash}.patch"));
            assert_eq!(parts[0].content, patch);
            assert!(config.patch_parts("  \n").is_empty());
        }
    }

    #[test]
    fn commit_mode_validates_hash_and_arguments() {
        fn parse_err(args: &[&str]) -> String {
            let error = Config::parse(args.iter().map(|arg| arg.to_string())).unwrap_err();
            format!("{error:?}")
        }

        let non_hex = "g".repeat(7);
        for args in [
            vec!["owner/repo", "--commit", "xyz"],
            vec!["owner/repo", "--commit", "abc"],
            vec!["owner/repo", "--commit", non_hex.as_str()],
        ] {
            assert!(parse_err(&args).contains("InvalidCommitHash"));
        }

        assert!(parse_err(&["owner/repo", "--commit"]).contains("MissingOptionValue"));
        assert!(parse_err(&["owner", "repo", "42", "--commit", "b430113"])
            .contains("InvalidCommitArguments"));
        assert!(parse_err(&["--commit", "b430113"]).contains("InvalidCommitArguments"));
        assert!(
            parse_err(&["owner/repo", "42", "--commit", "b430113", "--squash"])
                .contains("CommitWithSquash")
        );
    }

    #[test]
    fn gitlab_merge_request_downloads_per_commit_patches() {
        let config = config(&["--gitlab", "zitzhen/patchsplit", "1"]);
        assert_eq!(config.platform, Platform::GitLab);
        assert_eq!(config.project, "zitzhen/patchsplit");
        assert_eq!(
            config.patch_url(),
            "https://gitlab.com/zitzhen/patchsplit/-/merge_requests/1.patch"
        );
        let patch = format!(
            "From {} Mon Sep 17 00:00:00 2001\nSubject: [PATCH 1/2] First\n\nfirst\nFrom {} Mon Sep 17 00:00:00 2001\nSubject: [PATCH 2/2] Second\n\nsecond\n",
            "1".repeat(40),
            "2".repeat(40)
        );
        assert_eq!(config.patch_parts(&patch).len(), 2);
    }

    #[test]
    fn gitlab_squash_writes_one_mr_named_diff() {
        let config = config(&["--gitlab", "zitzhen/patchsplit", "1", "--squash", "--force"]);
        assert_eq!(config.project, "zitzhen/patchsplit");
        assert_eq!(
            config.patch_url(),
            "https://gitlab.com/zitzhen/patchsplit/-/merge_requests/1.diff"
        );
        let diff = "diff --git a/file b/file\n--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new\n";
        let parts = config.patch_parts(diff);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].subject, "MR #1");
        assert_eq!(parts[0].filename, "mr-1.patch");
        assert_eq!(parts[0].content, diff);
    }

    #[test]
    fn gitlab_commit_mode_downloads_a_single_commit_patch() {
        const FULL_HASH: &str = "fafbad69af7507f41e786aa6685b2fa29716c85f";
        let project = "zitzhen/patchsplit";
        let full_commit_option = format!("--commit={FULL_HASH}");
        let cases: Vec<(Vec<&str>, &str)> = vec![
            (vec!["--gitlab", project, "--commit", "fafbad6"], "fafbad6"),
            (
                vec!["--gitlab", "zitzhen", "patchsplit", "--commit", "de9ea1a"],
                "de9ea1a",
            ),
            (vec!["--gitlab", project, &full_commit_option], FULL_HASH),
            (
                vec![
                    "--gitlab",
                    "group/subgroup",
                    "project",
                    "--commit",
                    "de9ea1a",
                ],
                "de9ea1a",
            ),
        ];
        let patch = format!(
            "From {FULL_HASH} Mon Sep 17 00:00:00 2001\nSubject: [PATCH] One\n\ndiff --git a/a b/a\n"
        );

        for (args, hash) in cases {
            let config = config(&args);
            assert_eq!(config.platform, Platform::GitLab);
            assert_eq!(
                config.patch_url(),
                format!(
                    "https://gitlab.com/{}/-/commit/{hash}.patch",
                    config.project
                )
            );
            let parts = config.patch_parts(&patch);
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].filename, format!("{hash}.patch"));
        }
    }

    #[test]
    fn gitlab_mode_validates_project_numbers_and_arguments() {
        fn parse_err(args: &[&str]) -> String {
            let error = Config::parse(args.iter().map(|arg| arg.to_string())).unwrap_err();
            format!("{error:?}")
        }

        assert!(parse_err(&["--gitlab", "project-without-namespace", "363"])
            .contains("InvalidGitLabProject"));
        assert!(parse_err(&["--gitlab", "group//project", "363"]).contains("InvalidGitLabProject"));
        assert!(parse_err(&["--gitlab", "group/project", "0"]).contains("InvalidMergeRequest"));
        assert!(parse_err(&["--gitlab", "group/project", "abc"]).contains("InvalidMergeRequest"));
        assert!(parse_err(&["--gitlab", "363"]).contains("InvalidGitLabArguments"));
        assert!(parse_err(&["--gitlab"]).contains("InvalidGitLabArguments"));
        assert!(parse_err(&["--gitlab", "--commit", "de9ea1a"])
            .contains("InvalidGitLabCommitArguments"));
        assert!(parse_err(&["--gitlab", "group/project", "--commit", "xyz"])
            .contains("InvalidCommitHash"));
        assert!(parse_err(&[
            "--gitlab",
            "group/project",
            "363",
            "--commit",
            "de9ea1a",
            "--squash"
        ])
        .contains("CommitWithSquash"));
    }
}
