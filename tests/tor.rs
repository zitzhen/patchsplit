#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Workspace(PathBuf);

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const PROJECT: &str = "zitzhen/patchsplit";
const PR: &str = "16";

fn pr_patch_body() -> String {
    format!(
        "From {first} Mon Sep 17 00:00:00 2001\n\
From: Test <test@example.com>\n\
Subject: [PATCH 1/2] First\n\
\n\
diff --git a/a.txt b/a.txt\n\
new file mode 100644\n\
--- /dev/null\n\
+++ b/a.txt\n\
@@ -0,0 +1 @@\n\
+a\n\
From {second} Mon Sep 17 00:00:00 2001\n\
From: Test <test@example.com>\n\
Subject: [PATCH 2/2] Second\n\
\n\
diff --git a/b.txt b/b.txt\n\
new file mode 100644\n\
--- /dev/null\n\
+++ b/b.txt\n\
@@ -0,0 +1 @@\n\
+b\n",
        first = "1".repeat(40),
        second = "2".repeat(40)
    )
}

/// A fake curl that records how it was invoked and replays a canned patch.
const FAKE_CURL: &str = r#"#!/bin/sh
socks=""
proxy=""
url=""
previous=""
for arg do
  if [ "$previous" = "--socks5-hostname" ]; then proxy="$arg"; fi
  if [ "$arg" = "--socks5-hostname" ]; then socks="yes"; fi
  url="$arg"
  previous="$arg"
done
printf 'socks=%s\nproxy=%s\nurl=%s\n' "$socks" "$proxy" "$url" > "$PATCHSPLIT_TEST_SUMMARY"
if [ "$PATCHSPLIT_TEST_FAIL" = "tor" ]; then
  echo "curl: (7) Failed to connect to 127.0.0.1 port 9050: Connection refused" >&2
  exit 7
fi
cat "$PATCHSPLIT_TEST_PATCH"
"#;

#[test]
fn tor_cli_routes_the_download_through_a_socks5_proxy() {
    let root = Workspace(std::env::temp_dir().join(format!(
        "patchsplit-tor-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )));
    let bin = root.0.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let patch = root.0.join("pr.patch");
    let summary = root.0.join("summary");
    fs::write(&patch, pr_patch_body()).unwrap();

    // Substitute only the HTTP boundary; exercise the real CLI end to end.
    let curl = bin.join("curl");
    fs::write(&curl, FAKE_CURL).unwrap();
    fs::set_permissions(&curl, fs::Permissions::from_mode(0o755)).unwrap();
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(paths).unwrap();

    let run = |args: &[&str]| -> Output {
        Command::new(env!("CARGO_BIN_EXE_patchsplit"))
            .args(args)
            .env("PATH", &path)
            .env("PATCHSPLIT_TEST_PATCH", &patch)
            .env("PATCHSPLIT_TEST_SUMMARY", &summary)
            .env("PATCHSPLIT_LANGUAGE", "C")
            .output()
            .unwrap()
    };
    let recorded = || fs::read_to_string(&summary).unwrap();

    // A bare --tor uses Tor's default SOCKS5 endpoint.
    let tor_dir = root.0.join("tor");
    let result = run(&[PROJECT, PR, "--tor", "--out", tor_dir.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        recorded(),
        format!(
            "socks=yes\nproxy=127.0.0.1:9050\nurl=https://github.com/{PROJECT}/pull/{PR}.patch\n"
        )
    );
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("127.0.0.1:9050"),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let mut names: Vec<String> = fs::read_dir(&tor_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    assert_eq!(names, vec!["0001-first.patch", "0002-second.patch"]);

    // --tor=<host:port> selects another SOCKS5 endpoint, e.g. Tor Browser.
    let custom_dir = root.0.join("custom");
    let result = run(&[
        PROJECT,
        PR,
        "--tor=127.0.0.1:9150",
        "--out",
        custom_dir.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        recorded(),
        format!(
            "socks=yes\nproxy=127.0.0.1:9150\nurl=https://github.com/{PROJECT}/pull/{PR}.patch\n"
        )
    );

    // Without --tor the download must not be tunnelled through a proxy.
    let direct_dir = root.0.join("direct");
    let result = run(&[PROJECT, PR, "--out", direct_dir.to_str().unwrap()]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        recorded(),
        format!(
            "socks=\nproxy=\nurl=https://github.com/{PROJECT}/pull/{PR}.patch\n"
        )
    );

    // An unreachable Tor proxy fails with a hint instead of a bare curl error.
    let failure = Command::new(env!("CARGO_BIN_EXE_patchsplit"))
        .args([PROJECT, PR, "--tor"])
        .env("PATH", &path)
        .env("PATCHSPLIT_TEST_PATCH", &patch)
        .env("PATCHSPLIT_TEST_SUMMARY", &summary)
        .env("PATCHSPLIT_TEST_FAIL", "tor")
        .env("PATCHSPLIT_LANGUAGE", "C")
        .output()
        .unwrap();
    assert_eq!(failure.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&failure.stderr);
    assert!(
        stderr.contains("could not reach the Tor SOCKS5 proxy at 127.0.0.1:9050"),
        "{stderr}"
    );
    assert!(stderr.contains("--tor=<host:port>"), "{stderr}");
}
