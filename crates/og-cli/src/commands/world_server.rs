//! `og world-server`: the shared-world server (og-shared, docs/design.md
//! §2.12), standalone on any host; `og world-server load`: the load test's
//! bot swarm against one, over WebSocket or WebTransport, optionally
//! shaped; `og world-server token`: an HS256 token for a self-hosted server,
//! as a game's own token endpoint would sign it; `og world-server cert`: a
//! self-signed WebTransport certificate a browser takes by its hash.

use og_shared::auth::{Rules, Verifier, parse_jwks, sign_hs256};
use og_shared::bots::{self, LoadDesc};
use og_shared::config::Config;
use og_shared::server::{KeySource, Server, load_jwks, serve_metrics};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const USAGE: &str = "usage: og world-server [--listen ADDR] [--wt-listen ADDR --cert F --key F] [--config F] [--auth none|hs256|jwks] [--dev]
         [--secret-file F] [--jwks URL|FILE] [--audience GAME] [--issuer ISS] [--max-lifetime S] [--max-players N] [--metrics ADDR] [--quiet]
       og world-server load [--url ws://HOST:PORT/ | https://HOST:PORT/ [--cert-hash HEX]] [--bots N] [--area M] [--centre X,Z]
         [--areas N --spacing M] [--speed MPS] [--warmup S] [--seconds S] [--secret-file F --audience GAME] [--world W] [--batch N]
         [--latency MS] [--loss PCT] [--metrics HOST:PORT] [--gate] [--json]
       og world-server cert --out-cert F --out-key F [--name HOST]... [--days N]
       og world-server token --secret-file F --audience GAME --sub PLAYER [--name N] [--world W] [--ttl S]";

/// The design's budgets (docs/design.md §2.12): what `load --gate` holds a run to.
pub const DOWN_BUDGET: f64 = 8.0 * 1024.0;
pub const UP_BUDGET: f64 = 1024.0;
pub const TICK_P99_BUDGET_S: f64 = 0.010;

fn fail(m: impl std::fmt::Display) -> ExitCode {
    eprintln!("og world-server: {m}\n{USAGE}");
    ExitCode::from(2)
}

/// `--flag value` pairs and bare flags.
fn flags<'a>(args: &[&'a str], bare: &[&str]) -> Result<Vec<(&'a str, &'a str)>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i];
        if bare.contains(&a) {
            out.push((a, ""));
            i += 1;
        } else if a.starts_with("--") && i + 1 < args.len() {
            out.push((a, args[i + 1]));
            i += 2;
        } else {
            return Err(format!("unexpected `{a}`"));
        }
    }
    Ok(out)
}

fn read_secret(path: &str) -> Result<Vec<u8>, String> {
    let s = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let s = s.trim_ascii().to_vec();
    if s.len() < 32 {
        return Err(format!(
            "the secret in {path} is {} bytes; use at least 32",
            s.len()
        ));
    }
    Ok(s)
}

pub fn run(args: &[&str]) -> ExitCode {
    match args {
        ["load", rest @ ..] => load(rest),
        ["token", rest @ ..] => token(rest),
        ["cert", rest @ ..] => cert(rest),
        ["--help" | "-h"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => serve(args),
    }
}

fn serve(args: &[&str]) -> ExitCode {
    let f = match flags(args, &["--dev", "--quiet"]) {
        Ok(f) => f,
        Err(e) => return fail(e),
    };
    let get = |k: &str| f.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
    let dev = get("--dev").is_some();
    let mut config = match get("--config") {
        Some(p) => match std::fs::read_to_string(p)
            .map_err(|e| e.to_string())
            .and_then(|t| Config::parse(&t))
        {
            Ok(c) => c,
            Err(e) => return fail(format!("{p}: {e}")),
        },
        None => Config::default(),
    };
    if let Some(n) = get("--max-players") {
        match n.parse() {
            Ok(n) if n > 0 => config.max_players = n,
            _ => return fail("--max-players takes a whole number above 0"),
        }
    }
    let listen: SocketAddr = match get("--listen").unwrap_or("127.0.0.1:47910").parse() {
        Ok(a) => a,
        Err(_) => return fail("--listen takes an address such as 0.0.0.0:47910"),
    };
    let max_lifetime_s = match get("--max-lifetime").map(str::parse::<i64>) {
        Some(Ok(n)) if n > 0 => Some(n),
        Some(_) => return fail("--max-lifetime takes seconds above 0"),
        None => None,
    };
    let audience = get("--audience").map(|a| Rules {
        audience: a.into(),
        issuer: get("--issuer").map(str::to_string),
        max_lifetime_s,
    });
    let (verifier, keys) = match get("--auth").unwrap_or(if dev { "none" } else { "" }) {
        "none" if dev => (Verifier::None, KeySource::None),
        "none" => return fail("--auth none is for local development only: add --dev"),
        "hs256" => match (get("--secret-file"), audience) {
            (Some(p), Some(a)) => match read_secret(p) {
                Ok(s) => (Verifier::hs256(&s, a), KeySource::None),
                Err(e) => return fail(e),
            },
            _ => return fail("--auth hs256 takes --secret-file and --audience"),
        },
        "jwks" => match (get("--jwks"), audience) {
            (Some(j), Some(a)) => {
                let src = if j.starts_with("https://") || j.starts_with("http://") {
                    KeySource::Url(j.into())
                } else {
                    KeySource::File(j.into())
                };
                match load_jwks(&src).and_then(|t| parse_jwks(&t)) {
                    Ok(keys) => (Verifier::Jwks { keys, rules: a }, src),
                    Err(e) => return fail(e),
                }
            }
            _ => return fail("--auth jwks takes --jwks and --audience"),
        },
        "" => {
            return fail(
                "name a verifier: --auth hs256 or --auth jwks (or --dev for local development)",
            );
        }
        other => return fail(format!("no verifier `{other}`: none, hs256 or jwks")),
    };
    let metrics_addr = match get("--metrics").map(str::parse::<SocketAddr>) {
        Some(Ok(a)) => Some(a),
        Some(Err(_)) => return fail("--metrics takes an address such as 127.0.0.1:9910"),
        None => None,
    };
    let quiet = get("--quiet").is_some();
    let wt = match (get("--wt-listen"), get("--cert"), get("--key")) {
        (None, None, None) => None,
        (Some(a), Some(c), Some(k)) => {
            let addr: SocketAddr = match a.parse() {
                Ok(a) => a,
                Err(_) => return fail("--wt-listen takes an address such as 0.0.0.0:47911"),
            };
            let read =
                |f: &str| std::fs::read_to_string(f).map_err(|e| format!("cannot read {f}: {e}"));
            match (read(c), read(k)) {
                (Ok(c), Ok(k)) => Some((addr, c, k)),
                (Err(e), _) | (_, Err(e)) => return fail(e),
            }
        }
        _ => return fail("WebTransport takes --wt-listen, --cert and --key together"),
    };
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("a tokio runtime");
    rt.block_on(async move {
        let auth = verifier.name();
        let mut server = match Server::start(listen, config.clone(), verifier, keys, !quiet).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("og world-server: cannot listen on {listen}: {e}");
                return ExitCode::FAILURE;
            }
        };
        if dev {
            eprintln!("og world-server: WARNING: --dev: tokens are not checked (--auth {auth}); never expose this server");
        }
        println!(
            "world-server: ws://{} (auth {auth}, {} m cells, radius {}, {} in sight, {} Hz, {} players)",
            server.addr, config.cell_size, config.radius, config.max_visible, config.tick_hz, config.max_players
        );
        if let Some((addr, cert, key)) = &wt {
            match server.serve_webtransport(*addr, cert, key) {
                Ok(a) => println!("world-server: https://{a}/ (WebTransport)"),
                Err(e) => {
                    eprintln!("og world-server: cannot serve WebTransport on {addr}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        if let Some(a) = metrics_addr {
            match serve_metrics(a, server.metrics.clone()).await {
                Ok(m) => println!("world-server: metrics http://{m}/metrics"),
                Err(e) => eprintln!("og world-server: cannot serve metrics on {a}: {e}"),
            }
        }
        wait_for_signal().await;
        println!("world-server: draining");
        server.drain("the server is shutting down");
        server.drained(Duration::from_secs(30)).await;
        println!("world-server: stopped");
        ExitCode::SUCCESS
    })
}

async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("SIGTERM");
        tokio::select! {
            _ = term.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

/// Read `og_shared_<name>` from a metrics endpoint's text.
fn scrape(addr: &str) -> Result<Vec<(String, f64)>, String> {
    let mut s = std::net::TcpStream::connect(addr)
        .map_err(|e| format!("cannot reach the metrics at {addr}: {e}"))?;
    s.set_read_timeout(Some(Duration::from_secs(5))).ok();
    write!(
        s,
        "GET /metrics HTTP/1.1\r\nhost: {addr}\r\nconnection: close\r\n\r\n"
    )
    .map_err(|e| e.to_string())?;
    let mut text = String::new();
    s.read_to_string(&mut text).map_err(|e| e.to_string())?;
    Ok(text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let (k, v) = l.split_once(' ')?;
            Some((
                k.strip_prefix("og_shared_")?.to_string(),
                v.trim().parse().ok()?,
            ))
        })
        .collect())
}

/// Two scrapes of a server's metrics: what it did between them.
struct Window {
    a: Vec<(String, f64)>,
    b: Vec<(String, f64)>,
}

impl Window {
    fn new(a: &[(String, f64)], b: &[(String, f64)]) -> Window {
        Window {
            a: a.to_vec(),
            b: b.to_vec(),
        }
    }

    fn get(v: &[(String, f64)], n: &str) -> f64 {
        v.iter().find(|(k, _)| k == n).map_or(0.0, |(_, x)| *x)
    }

    fn last(&self, n: &str) -> f64 {
        Window::get(&self.b, n)
    }

    fn delta(&self, n: &str) -> f64 {
        Window::get(&self.b, n) - Window::get(&self.a, n)
    }

    /// The histogram `name`'s quantile `q` in the window, seconds: the upper
    /// bound of the bucket it falls in (`None` with no ticks, infinite past
    /// the last bound).
    fn quantile(&self, name: &str, q: f64) -> Option<f64> {
        let prefix = format!("{name}_bucket{{le=\"");
        let mut buckets: Vec<(f64, f64)> = self
            .b
            .iter()
            .filter_map(|(k, _)| {
                let le = k.strip_prefix(&prefix)?.strip_suffix("\"}")?;
                let bound = if le == "+Inf" {
                    f64::INFINITY
                } else {
                    le.parse().ok()?
                };
                Some((bound, self.delta(k)))
            })
            .collect();
        buckets.sort_by(|x, y| x.0.total_cmp(&y.0));
        let count = buckets.last()?.1;
        if count <= 0.0 {
            return None;
        }
        buckets
            .iter()
            .find(|(_, c)| *c >= q * count)
            .map(|(b, _)| *b)
    }

    fn json(&self) -> serde_json::Value {
        let q = |n: &str, p: f64| self.quantile(n, p).map(|t| t * 1e3);
        serde_json::json!({
            "players": self.last("players"),
            "tickMs": { "p50": q("tick_seconds", 0.5), "p99": q("tick_seconds", 0.99), "max": q("tick_seconds", 1.0) },
            "stepMs": { "p50": q("step_seconds", 0.5), "p99": q("step_seconds", 0.99), "max": q("step_seconds", 1.0) },
            "ticks": self.delta("tick_seconds_count"),
            "slowClosed": self.delta("slow_closed_total"),
            "rejected": self.delta("rejected_total"),
            "corrections": self.delta("corrections_total"),
            "bytesOut": self.delta("bytes_out_total"),
            "bytesIn": self.delta("bytes_in_total"),
        })
    }
}

fn load(args: &[&str]) -> ExitCode {
    let f = match flags(args, &["--gate", "--json"]) {
        Ok(f) => f,
        Err(e) => return fail(e),
    };
    let get = |k: &str| f.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
    fn num<T: std::str::FromStr>(v: Option<&str>, d: T, flag: &str) -> Result<T, String> {
        v.map_or(Ok(d), |v| {
            v.parse().map_err(|_| format!("{flag} takes a number"))
        })
    }
    let parsed = (|| -> Result<LoadDesc, String> {
        let secret = get("--secret-file").map(read_secret).transpose()?;
        let centre = match get("--centre") {
            Some(c) => {
                let v: Vec<f64> = c
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<_, _>>()
                    .map_err(|_| "--centre takes X,Z")?;
                if v.len() != 2 {
                    return Err("--centre takes X,Z".into());
                }
                [v[0], v[1]]
            }
            None => [0.0, 0.0],
        };
        Ok(LoadDesc {
            url: get("--url").unwrap_or("ws://127.0.0.1:47910/").into(),
            bots: num(get("--bots"), 200, "--bots")?,
            area: num(get("--area"), 640.0, "--area")?,
            centre,
            areas: num(get("--areas"), 1, "--areas")?,
            spacing: num(get("--spacing"), 4096.0, "--spacing")?,
            speed: num(get("--speed"), 4.0, "--speed")?,
            warmup: num(get("--warmup"), 5.0, "--warmup")?,
            seconds: num(get("--seconds"), 20.0, "--seconds")?,
            audience: get("--audience").unwrap_or("").into(),
            secret,
            world: get("--world").unwrap_or("main").into(),
            batch: num(get("--batch"), 50, "--batch")?,
            cert_hashes: get("--cert-hash")
                .map(og_shared::wt::parse_hash)
                .transpose()?
                .into_iter()
                .collect(),
            latency_ms: match num(get("--latency"), 0.0, "--latency")? {
                l if (0.0..=2000.0).contains(&l) => l,
                _ => return Err("--latency takes milliseconds from 0 to 2000".into()),
            },
            loss: match num(get("--loss"), 0.0, "--loss")? {
                l if (0.0..=50.0).contains(&l) => l / 100.0,
                _ => return Err("--loss takes a percentage from 0 to 50".into()),
            },
            seed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(1, |d| d.as_nanos() as u64),
        })
    })();
    let desc = match parsed {
        Ok(d) => d,
        Err(e) => return fail(e),
    };
    // Every bot joins within warmup: 50 ms a batch.
    let joining = (desc.bots / desc.batch.max(1)) as f64 * 0.05;
    let desc = LoadDesc {
        warmup: desc.warmup + joining,
        ..desc
    };
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("a tokio runtime");
    // The server's counters at the start and the end of the measured time:
    // their difference is what the server did while the bots were measured.
    let scrapes = get("--metrics").map(|addr| {
        let (addr, warmup, seconds) = (addr.to_string(), desc.warmup, desc.seconds);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs_f64(warmup));
            let first = scrape(&addr);
            std::thread::sleep(Duration::from_secs_f64((seconds - 0.5).max(0.1)));
            (first, scrape(&addr))
        })
    });
    let reps = rt.block_on(bots::run(desc.clone()));
    let s = bots::summarize(&reps);
    let window = match scrapes.map(|t| t.join().expect("the scrape thread")) {
        Some((Ok(a), Ok(b))) => Some(Window::new(&a, &b)),
        Some((a, b)) => {
            for e in [a.err(), b.err()].into_iter().flatten() {
                eprintln!("og world-server load: {e}");
            }
            None
        }
        None => None,
    };
    let tick_p99 = window
        .as_ref()
        .and_then(|w| w.quantile("tick_seconds", 0.99));
    let mut failures = Vec::new();
    if get("--gate").is_some() {
        if s.joined < s.bots {
            failures.push(format!(
                "{} of {} bots joined ({:?})",
                s.joined, s.bots, s.refused
            ));
        }
        if s.down[1] > DOWN_BUDGET {
            failures.push(format!(
                "down p95 {:.0} B/s is over {DOWN_BUDGET} B/s",
                s.down[1]
            ));
        }
        if s.up[1] > UP_BUDGET {
            failures.push(format!("up p95 {:.0} B/s is over {UP_BUDGET} B/s", s.up[1]));
        }
        if s.errors > 0 {
            failures.push(format!("{} messages did not decode", s.errors));
        }
        match tick_p99 {
            Some(t) if t > TICK_P99_BUDGET_S => {
                failures.push(format!("server tick p99 ≤ {:.1} ms is over 10 ms", t * 1e3))
            }
            None => failures.push("no server tick time: pass --metrics HOST:PORT".into()),
            _ => {}
        }
        if let Some(w) = &window
            && w.delta("slow_closed_total") > 0.0
        {
            failures.push(format!(
                "the server closed {} connections that fell behind",
                w.delta("slow_closed_total")
            ));
        }
    }
    if get("--json").is_some() {
        let j = serde_json::json!({
            "bots": s.bots, "joined": s.joined, "refused": s.refused,
            "transport": if desc.url.starts_with("https://") { "wt" } else { "ws" },
            "shaping": { "latencyMs": desc.latency_ms, "lossPercent": desc.loss * 100.0 },
            "delivery": {
                "delivered": s.delivered,
                "addedMs": { "p50": s.added_ms[0], "p99": s.added_ms[1] },
                "gapMs": { "p50": s.gap_ms[0], "p99": s.gap_ms[1], "max": s.gap_ms[2] },
            },
            "area": desc.area, "areas": desc.areas, "speed": desc.speed, "seconds": desc.seconds,
            "downBytesPerSecond": { "mean": s.down[0], "p95": s.down[1], "max": s.down[2] },
            "upBytesPerSecond": { "mean": s.up[0], "p95": s.up[1], "max": s.up[2] },
            "inSight": { "mean": s.in_sight_mean, "max": s.in_sight_max }, "othersMax": s.others_max,
            "updatesPerSecond": s.updates_per_s, "corrections": s.corrections, "errors": s.errors, "stale": s.stale, "moved": s.moved,
            "server": window.as_ref().map(Window::json),
            "gate": if get("--gate").is_some() { serde_json::json!({ "pass": failures.is_empty(), "failures": failures }) } else { serde_json::Value::Null },
        });
        println!("{j:#}");
    } else {
        println!(
            "bots        {} joined of {} ({} areas of {} m square at {} m/s, {} s measured)",
            s.joined, s.bots, desc.areas, desc.area, desc.speed, desc.seconds
        );
        println!(
            "down        mean {:.0} B/s  p95 {:.0} B/s  max {:.0} B/s   (budget {DOWN_BUDGET} B/s)",
            s.down[0], s.down[1], s.down[2]
        );
        println!(
            "up          mean {:.0} B/s  p95 {:.0} B/s  max {:.0} B/s   (budget {UP_BUDGET} B/s)",
            s.up[0], s.up[1], s.up[2]
        );
        println!(
            "in sight    mean {:.1}  max {}  (others counted, max {})",
            s.in_sight_mean, s.in_sight_max, s.others_max
        );
        println!(
            "updates     {:.0} a second per client; {} corrections, {} decode errors, {} stale",
            s.updates_per_s, s.corrections, s.errors, s.stale
        );
        println!(
            "delivery    {} over {} ms added RTT, {}% loss: {:.1}% of snapshots arrived; delay past the fastest p50 {:.1} ms p99 {:.1} ms; gaps p50 {:.1} ms p99 {:.1} ms max {:.1} ms",
            if desc.url.starts_with("https://") {
                "WebTransport"
            } else {
                "WebSocket"
            },
            desc.latency_ms,
            desc.loss * 100.0,
            s.delivered * 100.0,
            s.added_ms[0],
            s.added_ms[1],
            s.gap_ms[0],
            s.gap_ms[1],
            s.gap_ms[2]
        );
        if let Some(w) = &window {
            let q = |n: &str, p: f64| {
                w.quantile(n, p)
                    .map_or("-".into(), |t| format!("≤ {:.1}", t * 1e3))
            };
            println!(
                "server      tick p50 {} ms  p99 {} ms  (step alone p50 {} ms  p99 {} ms; budget p99 10 ms); {} players, {} slow closes",
                q("tick_seconds", 0.5),
                q("tick_seconds", 0.99),
                q("step_seconds", 0.5),
                q("step_seconds", 0.99),
                w.last("players"),
                w.delta("slow_closed_total")
            );
        }
        for r in &s.refused {
            println!("refused     {r}");
        }
        if get("--gate").is_some() {
            println!(
                "gate        {}",
                if failures.is_empty() {
                    "pass".to_string()
                } else {
                    format!("FAIL: {}", failures.join("; "))
                }
            );
        }
    }
    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn cert(args: &[&str]) -> ExitCode {
    let mut names = Vec::new();
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--name" && i + 1 < args.len() {
            names.push(args[i + 1].to_string());
            i += 2;
        } else {
            rest.push(args[i]);
            i += 1;
        }
    }
    let f = match flags(&rest, &[]) {
        Ok(f) => f,
        Err(e) => return fail(e),
    };
    let get = |k: &str| f.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
    let (Some(out_cert), Some(out_key)) = (get("--out-cert"), get("--out-key")) else {
        return fail("cert takes --out-cert and --out-key");
    };
    if names.is_empty() {
        names = vec!["localhost".into(), "127.0.0.1".into()];
    }
    let days = match get("--days").unwrap_or("10").parse() {
        Ok(d) => d,
        Err(_) => return fail("--days takes 1 to 14"),
    };
    let c = match og_shared::cert::self_signed(&names, days) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    if let Err(e) =
        std::fs::write(out_cert, &c.cert_pem).and_then(|_| std::fs::write(out_key, &c.key_pem))
    {
        return fail(e);
    }
    // The hash a page passes as serverCertificateHashes and `load --cert-hash` takes.
    println!(
        "{}",
        c.sha256
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    ExitCode::SUCCESS
}

fn token(args: &[&str]) -> ExitCode {
    let f = match flags(args, &[]) {
        Ok(f) => f,
        Err(e) => return fail(e),
    };
    let get = |k: &str| f.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
    let (Some(secret), Some(aud), Some(sub)) =
        (get("--secret-file"), get("--audience"), get("--sub"))
    else {
        return fail("token takes --secret-file, --audience and --sub");
    };
    let secret = match read_secret(secret) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let ttl: u64 = match get("--ttl").unwrap_or("300").parse() {
        Ok(t) => t,
        Err(_) => return fail("--ttl takes seconds"),
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut claims = serde_json::json!({ "aud": aud, "sub": sub, "iat": now, "exp": now + ttl });
    if let Some(n) = get("--name") {
        claims["name"] = n.into();
    }
    if let Some(w) = get("--world") {
        claims["world"] = w.into();
    }
    println!("{}", sign_hs256(&secret, &claims));
    ExitCode::SUCCESS
}
