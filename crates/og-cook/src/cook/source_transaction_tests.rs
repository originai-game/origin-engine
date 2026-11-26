//! Persistence contracts, exercised through commit on real files. The test-only
//! checkpoints inject IO failure/process death, not an alternate filesystem.
use super::{CHECKPOINT, FileEdit, SavedFile, commit};
use crate::cook::generator::source::{Edit, Source};
use crate::cook::test_dir::TempDir;
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

const A: &str = "export default { mass: 2 };\r\n";
const B: &str = "export default { mass: 7 };\n";

fn project() -> TempDir {
    let root = TempDir::new("source-transaction");
    fs::create_dir(root.join("content")).unwrap();
    fs::write(root.join("content/a.ts"), A).unwrap();
    fs::write(root.join("content/b.ts"), B).unwrap();
    root
}

fn edit(file: &str, old: &str, value: i32) -> FileEdit {
    let source = Source::new(file, old);
    FileEdit {
        file: file.into(),
        digest: source.digest(),
        text: source
            .patch(
                old,
                &[Edit {
                    path: "/mass".into(),
                    expected: Some(json!(if file.ends_with("a.ts") { 2 } else { 7 })),
                    value: json!(value),
                }],
            )
            .unwrap(),
    }
}

fn edits() -> Vec<FileEdit> {
    vec![edit("content/b.ts", B, 31), edit("content/a.ts", A, 13)]
}

fn assert_original(root: &Path) {
    assert_eq!(fs::read(root.join("content/a.ts")).unwrap(), A.as_bytes());
    assert_eq!(fs::read(root.join("content/b.ts")).unwrap(), B.as_bytes());
}

#[test]
fn batch_results_and_validation_overlay_are_deterministic() {
    let first = project();
    let second = project();
    let mut batch = edits();
    let saved = commit(&first, &batch, |overlay| {
        assert_original(&first);
        assert_eq!(
            overlay.iter().map(|e| e.file.as_str()).collect::<Vec<_>>(),
            ["content/a.ts", "content/b.ts"]
        );
        Ok(())
    })
    .unwrap();
    batch.reverse();
    assert_eq!(saved, commit(&second, &batch, |_| Ok(())).unwrap());
    for (result, expected) in saved.iter().zip(batch.iter()) {
        assert_eq!(
            result,
            &SavedFile {
                file: expected.file.clone(),
                digest: Source::new(&expected.file, &expected.text).digest()
            }
        );
        assert_eq!(
            fs::read(first.join(&result.file)).unwrap(),
            expected.text.as_bytes()
        );
    }
    let unchanged: Vec<_> = batch
        .into_iter()
        .map(|mut e| {
            e.digest = Source::new(&e.file, &e.text).digest();
            e
        })
        .collect();
    assert!(commit(&first, &unchanged, |_| Ok(())).unwrap().is_empty());
}

#[test]
fn stale_snapshot_and_validator_rejection_change_no_sources() {
    let root = project();
    let batch = edits();
    // Same-length disk edit: neither stat nor old field value can stand in for bytes.
    fs::write(root.join("content/b.ts"), B.replace('7', "9")).unwrap();
    assert_eq!(
        commit(&root, &batch, |_| panic!("stale batch reached validation")).unwrap_err()[0].code,
        "source-conflict"
    );
    assert_eq!(fs::read_to_string(root.join("content/a.ts")).unwrap(), A);
    assert_eq!(
        fs::read_to_string(root.join("content/b.ts")).unwrap(),
        B.replace('7', "9")
    );
    fs::write(root.join("content/b.ts"), B).unwrap();
    assert_eq!(
        commit(&root, &batch, |_| Err(vec![])).unwrap_err()[0].code,
        "source-validation"
    );
    assert_original(&root);

    // Simulate an uncooperative external writer while validation is running.
    assert_eq!(
        commit(&root, &batch, |_| {
            fs::write(root.join("content/b.ts"), "external edit").unwrap();
            Ok(())
        })
        .unwrap_err()[0]
            .code,
        "source-conflict"
    );
    assert_eq!(fs::read_to_string(root.join("content/a.ts")).unwrap(), A);
    assert_eq!(
        fs::read_to_string(root.join("content/b.ts")).unwrap(),
        "external edit"
    );
}

#[test]
fn paths_and_aliases_cannot_escape_or_duplicate_sources() {
    let root = project();
    let valid = edit("content/a.ts", A, 13);
    assert_eq!(
        commit(&root, &[valid.clone(), valid.clone()], |_| panic!()).unwrap_err()[0].code,
        "duplicate-source"
    );
    for path in [
        "../a.ts",
        "/a.ts",
        "content/../content/a.ts",
        "content//a.ts",
        "./content/a.ts",
        ".og/source-edit.lock",
        "C:\\a.ts",
    ] {
        let mut edit = valid.clone();
        edit.file = path.into();
        assert!(
            commit(&root, &[edit], |_| panic!()).is_err(),
            "accepted {path}"
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("content/a.ts"), root.join("alias.ts")).unwrap();
        std::os::unix::fs::symlink(root.join("content"), root.join("alias")).unwrap();
        for file in ["alias.ts", "alias/a.ts"] {
            let mut edit = valid.clone();
            edit.file = file.into();
            assert!(commit(&root, &[edit], |_| panic!()).is_err());
        }
    }
    assert_original(&root);
}

#[test]
fn mid_batch_failure_rolls_back_but_never_overwrites_an_external_edit() {
    for external in [false, true] {
        let root = project();
        CHECKPOINT.with_borrow_mut(|hook| {
            *hook = Some(Box::new(move |phase, path| {
                if phase == "replace" && path.ends_with("b.ts") {
                    if external {
                        fs::write(path.with_file_name("a.ts"), "external after replace")?;
                    }
                    return Err(io::Error::other("injected second replacement failure"));
                }
                Ok(())
            }))
        });
        let errors = commit(&root, &edits(), |_| Ok(())).unwrap_err();
        CHECKPOINT.with_borrow_mut(|hook| *hook = None);
        assert_eq!(errors[0].code, "source-io");
        assert_eq!(fs::read_to_string(root.join("content/b.ts")).unwrap(), B);
        if external {
            assert!(errors.iter().any(|e| e.code == "source-recovery-failed"));
            assert_eq!(
                fs::read_to_string(root.join("content/a.ts")).unwrap(),
                "external after replace"
            );
            assert!(
                commit(&root, &[], |_| panic!(
                    "unresolved recovery reached validation"
                ))
                .is_err()
            );
            assert_eq!(
                fs::read_to_string(root.join("content/a.ts")).unwrap(),
                "external after replace"
            );
            // Explicit external resolution permits recovery, not a silent
            // overwrite of the third party's bytes by the next save.
            fs::write(root.join("content/a.ts"), &edits()[1].text).unwrap();
            assert_eq!(
                commit(&root, &[], |_| panic!()).unwrap_err()[0].code,
                "recovered-transaction"
            );
            assert_original(&root);
        } else {
            assert_original(&root);
            assert_eq!(commit(&root, &edits(), |_| Ok(())).unwrap().len(), 2);
        }
    }
}

fn child(root: &Path, role: &str) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cook::source_transaction::tests::transaction_child",
            "--nocapture",
        ])
        .env("OG_SOURCE_TRANSACTION_ROOT", root)
        .env("OG_SOURCE_TRANSACTION_ROLE", role)
        .spawn()
        .unwrap()
}

fn wait(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

// Also runs alone in a fresh test process. No inherited in-process mutex can
// accidentally make the cross-process locking or crash-recovery contracts pass.
#[test]
fn transaction_child() {
    let Some(root) = std::env::var_os("OG_SOURCE_TRANSACTION_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let role = std::env::var("OG_SOURCE_TRANSACTION_ROLE").unwrap();
    if role.starts_with("crash-") {
        CHECKPOINT.with_borrow_mut(|hook| {
            *hook = Some(Box::new(move |phase, path| {
                if (role == "crash-intent" && phase == "journal")
                    || (role == "crash-middle" && phase == "replaced" && path.ends_with("a.ts"))
                    || (role == "crash-complete" && phase == "replaced" && path.ends_with("b.ts"))
                    || (role == "crash-rollback" && phase == "restored")
                {
                    std::process::exit(71);
                }
                if role == "crash-rollback" && phase == "replace" && path.ends_with("b.ts") {
                    return Err(io::Error::other("force rollback before process death"));
                }
                Ok(())
            }))
        });
        let _ = commit(root, &edits(), |_| Ok(()));
        panic!("crash checkpoint not reached");
    }
    fs::write(root.join(format!("{role}-started")), "").unwrap();
    let result = commit(root, &edits(), |_| {
        fs::write(root.join(format!("{role}-validating")), "").unwrap();
        if role == "first" {
            wait(&root.join("release-first"));
        }
        Ok(())
    });
    if role == "first" {
        assert_eq!(result.unwrap().len(), 2);
    } else {
        assert_eq!(result.unwrap_err()[0].code, "source-conflict");
    }
}

#[test]
fn project_lock_serializes_processes_and_checks_snapshot_after_waiting() {
    let root = project();
    let mut first = child(&root, "first");
    wait(&root.join("first-validating"));
    let mut second = child(&root, "second");
    wait(&root.join("second-started"));
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        second.try_wait().unwrap().is_none(),
        "second process did not wait for the project lock"
    );
    assert!(!root.join("second-validating").exists());
    fs::write(root.join("release-first"), "").unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    assert!(!root.join("second-validating").exists());
}

#[test]
fn interrupted_batches_recover_before_any_new_validation() {
    for role in [
        "crash-intent",
        "crash-middle",
        "crash-complete",
        "crash-rollback",
    ] {
        let root = project();
        assert_eq!(child(&root, role).wait().unwrap().code(), Some(71));
        let errors = commit(&root, &[], |_| {
            panic!("incoming save must be retried after recovery")
        })
        .unwrap_err();
        assert_eq!(
            errors[0].code, "recovered-transaction",
            "{role}: {errors:?}"
        );
        assert_original(&root);
        assert_eq!(commit(&root, &edits(), |_| Ok(())).unwrap().len(), 2);
    }
}
