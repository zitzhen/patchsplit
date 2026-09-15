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

const SHORT_HASH: &str = "b430113";
const FULL_HASH: &str = "b4301133226e5c3a464cff9649de0b321c0b0a2e";

fn patch_body() -> String {
    format!(
        "From {FULL_HASH} Mon Sep 17 00:00:00 2001\n\
From: Test <test@example.com>\n\
Subject: [PATCH] Single commit\n\
\n\
diff --git a/a.txt b/a.txt\n\
new file mode 100644\n\
--- /dev/null\n\
+++ b/a.txt\n\
@@ -0,0 +1 @@\n\
+a\n"
    )
}

#[test]
fn commit_cli_downloads_one_commit_patch_for_short_and_full_hash() {
    let root = Workspace(std::env::temp_dir().join(format!(
        "patchsplit-commit-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )));
    let bin = root.0.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let fixture = root.0.join("response.patch");
    fs::write(&fixture, patch_body()).unwrap();

    // Substitute only the HTTP boundary; exercise the real CLI end to end.
    let curl = bin.join("curl");
    fs::write(
        &curl,
        "#!/bin/sh\n\
for arg do url=\"$arg\"; done\n\
case \"$url\" in\n\
  'https://github.com/zitzhen/patchsplit/commit/b430113.patch') ;;\n\
  'https://github.com/zitzhen/patchsplit/commit/b4301133226e5c3a464cff9649de0b321c0b0a2e.patch') ;;\n\
  *) echo \"unexpected url: $url\" >&2; exit 22 ;;\n\
esac\n\
cat \"$PATCHSPLIT_TEST_PATCH\"\n",
    )
    .unwrap();
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
            .env("PATCHSPLIT_TEST_PATCH", &fixture)
            .env("PATCHSPLIT_LANGUAGE", "C")
            .output()
            .unwrap()
    };

    let short_dir = root.0.join("short");
    let result = run(&[
        "zitzhen/patchsplit",
        "-commit",
        SHORT_HASH,
        "--out",
        short_dir.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let short_patch = short_dir.join(format!("{SHORT_HASH}.patch"));
    assert_eq!(fs::read_dir(&short_dir).unwrap().count(), 1);
    assert_eq!(fs::read(&short_patch).unwrap(), patch_body().as_bytes());

    let full_dir = root.0.join("full");
    let result = run(&[
        "zitzhen",
        "patchsplit",
        "--commit",
        FULL_HASH,
        "--out",
        full_dir.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let full_patch = full_dir.join(format!("{FULL_HASH}.patch"));
    assert_eq!(fs::read(&full_patch).unwrap(), patch_body().as_bytes());

    let bad = run(&["zitzhen/patchsplit", "--commit", "xyz"]);
    assert_eq!(bad.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("hexadecimal characters")
    );

    let conflict = run(&[
        "zitzhen/patchsplit",
        "42",
        "--commit",
        SHORT_HASH,
        "--squash",
    ]);
    assert_eq!(conflict.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&conflict.stderr)
            .contains("--commit cannot be combined with --squash")
    );
}
