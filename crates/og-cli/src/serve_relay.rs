//! `og relay [--port P] [--host <ip>] [--latency MS] [--jitter MS] [--loss PCT]
//! [--seed N]`: a local WebSocket relay for rooms
//! (og-net's relay, docs/design.md §2.8), so native runs play together:
//! `og run --room ws://127.0.0.1:P/<room>` in two terminals (or two
//! playtest processes) join room `<room>` and their engines exchange
//! inputs and hashes through it (og.net's lockstep). It prints one line
//! when listening and one per join and departure, and runs until a signal.
//! `--latency`, `--jitter`, `--loss` and `--seed` make it a bad network
//! (og-net's `Shaping`): every frame held, unreliable ones dropped and
//! reordered, to test rollback and stalls against.

use og_net::relay::{DEFAULT_PORT, Relay, Shaping};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::ExitCode;
use std::time::Duration;

const USAGE: &str =
    "usage: og relay [--port P] [--host <ip>] [--latency MS] [--jitter MS] [--loss PCT] [--seed N]";

pub fn run(args: &[&str]) -> ExitCode {
    let mut port = DEFAULT_PORT;
    let mut host = IpAddr::V4(Ipv4Addr::LOCALHOST);
    let mut shaping = Shaping::default();
    let mut i = 0;
    while i < args.len() {
        let value = args.get(i + 1);
        match (args[i], value) {
            ("--port", Some(v)) => match v.parse() {
                Ok(p) => port = p,
                Err(_) => {
                    eprintln!("og relay: --port takes 0 (a free port) to 65535. {USAGE}");
                    return ExitCode::from(2);
                }
            },
            ("--host", Some(v)) => match v.parse() {
                Ok(h) => host = h,
                Err(_) => {
                    eprintln!("og relay: --host takes an IP address such as 0.0.0.0. {USAGE}");
                    return ExitCode::from(2);
                }
            },
            (flag @ ("--latency" | "--jitter" | "--loss"), Some(v)) => match v.parse::<u32>() {
                Ok(n) if flag != "--loss" || n <= 100 => match flag {
                    "--latency" => shaping.latency_ms = n,
                    "--jitter" => shaping.jitter_ms = n,
                    _ => shaping.loss_percent = n,
                },
                _ => {
                    eprintln!(
                        "og relay: {flag} takes a whole number ({}). {USAGE}",
                        if flag == "--loss" {
                            "percent, 0 to 100"
                        } else {
                            "milliseconds"
                        }
                    );
                    return ExitCode::from(2);
                }
            },
            ("--seed", Some(v)) => match v.parse() {
                Ok(n) => shaping.seed = n,
                Err(_) => {
                    eprintln!("og relay: --seed takes a whole number. {USAGE}");
                    return ExitCode::from(2);
                }
            },
            (other, _) => {
                eprintln!("og relay: unexpected `{other}`. {USAGE}");
                return ExitCode::from(2);
            }
        }
        i += 2;
    }
    let relay = match Relay::bind_shaped(SocketAddr::new(host, port), shaping) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("og relay: cannot listen on {host}:{port}: {e}; pick another --port");
            return ExitCode::FAILURE;
        }
    };
    println!(
        "relay: ws://{host}:{}/<room> (og run --room ws://{host}:{}/<room>)",
        relay.port(),
        relay.port()
    );
    if shaping != Shaping::default() {
        println!(
            "relay: shaped: {} ms latency, up to {} ms jitter, {} % of unreliable messages lost (seed {})",
            shaping.latency_ms, shaping.jitter_ms, shaping.loss_percent, shaping.seed
        );
    }
    loop {
        for line in relay.take_log() {
            println!("{line}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
