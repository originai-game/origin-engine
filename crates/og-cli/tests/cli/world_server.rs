use super::*;

/// A server started under a shell's low soft limit on open files raises it
/// to the hard limit, and when even that cannot hold its players it says so
/// and serves the players it can hold.
#[cfg(unix)]
#[test]
fn world_server_raises_the_open_file_limit() {
    use std::io::{BufRead, BufReader, Read};
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    let mut cmd = og();
    cmd.args([
        "world-server",
        "--dev",
        "--quiet",
        "--listen",
        "127.0.0.1:0",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    // SAFETY: setrlimit is async-signal-safe.
    unsafe {
        cmd.pre_exec(|| {
            let lim = libc::rlimit {
                rlim_cur: 256,
                rlim_max: 512,
            };
            if libc::setrlimit(libc::RLIMIT_NOFILE, &lim) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = cmd.spawn().unwrap();
    let mut banner = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut banner)
        .unwrap();
    #[cfg(target_os = "linux")]
    let limits = std::fs::read_to_string(format!("/proc/{}/limits", child.id())).unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    let mut err = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut err)
        .unwrap();
    assert!(
        err.contains(
            "2000 players need 2064 open files but this host allows 512, so it serves 448"
        ),
        "{err}"
    );
    assert!(banner.contains(", 448 players)"), "{banner}");
    #[cfg(target_os = "linux")]
    {
        let open = limits
            .lines()
            .find(|l| l.starts_with("Max open files"))
            .unwrap();
        assert_eq!(
            open.split_whitespace().collect::<Vec<_>>()[3..5],
            ["512", "512"],
            "{open}"
        );
    }
}
