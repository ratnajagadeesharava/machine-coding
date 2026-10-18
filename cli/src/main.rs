//! `play` - a rustlings-style runner: test, watch and track each playground problem separately.
//!
//! Usage (from the repo root):  cargo play <command> ...   (see `cargo play help`)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lang {
    Rust,
    Java,
}

impl Lang {
    fn name(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Java => "java",
        }
    }
    fn parse(s: &str) -> Option<Lang> {
        match s.to_ascii_lowercase().as_str() {
            "rust" | "rs" | "r" => Some(Lang::Rust),
            "java" | "j" => Some(Lang::Java),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    NotStarted,
    Failing,
    BuildError,
    Passing,
}

impl Status {
    fn key(self) -> &'static str {
        match self {
            Status::NotStarted => "not-started",
            Status::Failing => "failing",
            Status::BuildError => "build-error",
            Status::Passing => "passing",
        }
    }
    fn parse(s: &str) -> Status {
        match s {
            "failing" => Status::Failing,
            "build-error" => Status::BuildError,
            "passing" => Status::Passing,
            _ => Status::NotStarted,
        }
    }
    /// (coloured label, visible width)
    fn label(self) -> (String, usize) {
        match self {
            Status::NotStarted => (dim("- not started"), 13),
            Status::Failing => (red("x failing"), 9),
            Status::BuildError => (red("! build error"), 13),
            Status::Passing => (green("v passing"), 9),
        }
    }
}

// ---------- colours ----------

fn color_on() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}
fn paint(code: &str, s: &str) -> String {
    if color_on() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}
fn red(s: &str) -> String {
    paint("31", s)
}
fn green(s: &str) -> String {
    paint("32", s)
}
fn yellow(s: &str) -> String {
    paint("33", s)
}
fn dim(s: &str) -> String {
    paint("2", s)
}
fn bold(s: &str) -> String {
    paint("1", s)
}

// ---------- repo model ----------

struct Meta {
    title: String,
    difficulty: String,
    minutes: String,
    level: String,
}

struct Repo {
    root: PathBuf,
}

struct Problem {
    number: u32,
    slug: String,
}

impl Problem {
    fn crate_name(&self) -> String {
        format!("p{}", self.slug.replace('-', "_"))
    }
}

type Progress = HashMap<(String, String), Status>;

impl Repo {
    fn locate() -> Result<Repo, String> {
        let mut starts: Vec<PathBuf> = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            starts.push(cwd);
        }
        starts.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
        for start in starts {
            let mut dir = Some(start);
            while let Some(d) = dir {
                if d.join("rust").join("problems").is_dir() && d.join("java").join("problems").is_dir() {
                    let root = d.canonicalize().unwrap_or(d);
                    let shown = root.to_string_lossy().strip_prefix("\\\\?\\").unwrap_or(&root.to_string_lossy()).to_string();
                    return Ok(Repo {
                        root: PathBuf::from(shown),
                    });
                }
                dir = d.parent().map(|p| p.to_path_buf());
            }
        }
        Err("cannot find the playground repo (expected rust/problems and java/problems)".into())
    }

    fn problems(&self) -> Vec<Problem> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(self.root.join("rust").join("problems")) {
            for e in rd.flatten() {
                let slug = e.file_name().to_string_lossy().to_string();
                if let Some((n, _)) = slug.split_once('-') {
                    if let Ok(number) = n.parse::<u32>() {
                        out.push(Problem { number, slug });
                    }
                }
            }
        }
        out.sort_by_key(|p| p.number);
        out
    }

    fn metas(&self) -> HashMap<u32, Meta> {
        let mut map = HashMap::new();
        let text = fs::read_to_string(self.root.join("README.md")).unwrap_or_default();
        for line in text.lines() {
            if !line.starts_with('|') {
                continue;
            }
            let cols: Vec<&str> = line.trim_matches('|').split('|').map(|c| c.trim()).collect();
            if cols.len() < 5 {
                continue;
            }
            let Ok(n) = cols[0].parse::<u32>() else {
                continue;
            };
            let title = cols[1]
                .strip_prefix('[')
                .and_then(|t| t.split(']').next())
                .unwrap_or(cols[1])
                .to_string();
            map.insert(
                n,
                Meta {
                    title,
                    difficulty: cols[2].to_string(),
                    minutes: cols[3].replace(" min", "").trim().to_string(),
                    level: cols[4].to_string(),
                },
            );
        }
        map
    }

    fn problem_dir(&self, lang: Lang, p: &Problem) -> PathBuf {
        self.root.join(lang.name()).join("problems").join(&p.slug)
    }

    fn resolve(&self, query: &str) -> Result<Problem, String> {
        let all = self.problems();
        if let Ok(n) = query.parse::<u32>() {
            return all
                .into_iter()
                .find(|p| p.number == n)
                .ok_or_else(|| format!("no problem number {n}; try `cargo play list`"));
        }
        let q = query.to_ascii_lowercase();
        let mut hits: Vec<Problem> = all.into_iter().filter(|p| p.slug.contains(&q)).collect();
        match hits.len() {
            0 => Err(format!("no problem matches '{query}'; try `cargo play list`")),
            1 => Ok(hits.remove(0)),
            _ => Err(format!(
                "'{query}' is ambiguous: {}",
                hits.iter().map(|p| p.slug.clone()).collect::<Vec<_>>().join(", ")
            )),
        }
    }

    // ----- progress -----

    fn progress_file(&self) -> PathBuf {
        self.root.join(".playground").join("progress.txt")
    }

    fn load_progress(&self) -> Progress {
        let mut map = HashMap::new();
        for line in fs::read_to_string(self.progress_file()).unwrap_or_default().lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                map.insert((parts[0].to_string(), parts[1].to_string()), Status::parse(parts[2]));
            }
        }
        map
    }

    fn record(&self, lang: Lang, p: &Problem, st: Status) {
        let mut map = self.load_progress();
        map.insert((lang.name().to_string(), p.slug.clone()), st);
        let mut lines: Vec<String> = map.iter().map(|((l, s), st)| format!("{l} {s} {}", st.key())).collect();
        lines.sort();
        let _ = fs::create_dir_all(self.root.join(".playground"));
        let _ = fs::write(self.progress_file(), lines.join("\n") + "\n");
    }
}

// ---------- running tests ----------

struct RunOpts {
    lang: Lang,
    hidden: bool,
    strict: bool,
    verbose: bool,
}

struct Outcome {
    status: Status,
    passed: u32,
    failed: u32,
    skipped: u32,
    failures: Vec<String>,
    raw: String,
}

fn shell(program: &str, args: &[String], dir: &Path, strict: bool) -> std::io::Result<Output> {
    let mut cmd = if cfg!(windows) && program == "mvn" {
        let mut c = Command::new("cmd");
        c.args(["/C", "mvn"]);
        c
    } else {
        Command::new(program)
    };
    cmd.args(args).current_dir(dir);
    if strict {
        cmd.env("STRICT", "1");
    }
    cmd.env("NO_COLOR", "1");
    cmd.output()
}

fn run_tests(repo: &Repo, p: &Problem, o: &RunOpts) -> Result<Outcome, String> {
    let (program, args, dir): (&str, Vec<String>, PathBuf) = match o.lang {
        Lang::Rust => {
            let mut a = vec!["test".to_string(), "-p".into(), p.crate_name()];
            if o.hidden {
                a.push("--features".into());
                a.push("hidden".into());
            }
            a.push("--".into());
            a.push("--nocapture".into());
            ("cargo", a, repo.root.join("rust"))
        }
        Lang::Java => {
            let mut a = vec![
                "-B".to_string(),
                "-ntp".into(),
                "-pl".into(),
                format!("problems/{}", p.slug),
                "-am".into(),
            ];
            if o.hidden {
                a.push("-Phidden".into());
            }
            a.push("test".into());
            ("mvn", a, repo.root.join("java"))
        }
    };
    let out = shell(program, &args, &dir, o.strict).map_err(|e| {
        let hint = if program == "mvn" { " (is Maven installed and on PATH?)" } else { "" };
        format!("could not start `{program}`: {e}{hint}")
    })?;
    let raw = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if program == "mvn" && (raw.contains("'mvn' is not recognized") || raw.contains("mvn: command not found")) {
        return Err("Maven (`mvn`) is not installed or not on PATH; install Maven and a JDK 21 to run the Java side".into());
    }
    Ok(match o.lang {
        Lang::Rust => parse_rust(&raw, out.status.success()),
        Lang::Java => parse_java(&raw, out.status.success()),
    })
}

fn parse_rust(raw: &str, success: bool) -> Outcome {
    let (mut passed, mut failed, mut skipped) = (0u32, 0u32, 0u32);
    let mut failures = Vec::new();
    let mut saw_result = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with("test result:") {
            saw_result = true;
            passed += number_before(t, " passed");
            failed += number_before(t, " failed");
        } else if t.starts_with("test ") && t.ends_with("FAILED") {
            failures.push(t.trim_start_matches("test ").trim_end_matches(" ... FAILED").to_string());
        } else if t.contains("SKIPPED:") {
            skipped += 1;
        }
    }
    let status = if !saw_result {
        Status::BuildError
    } else if failed > 0 || !success {
        Status::Failing
    } else if skipped > 0 {
        Status::NotStarted
    } else {
        Status::Passing
    };
    Outcome {
        status,
        passed,
        failed,
        skipped,
        failures,
        raw: raw.to_string(),
    }
}

fn parse_java(raw: &str, success: bool) -> Outcome {
    let (mut run, mut failed, mut skipped) = (0u32, 0u32, 0u32);
    let mut failures = Vec::new();
    let mut saw = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.contains("Tests run:") && !t.contains("Time elapsed") {
            saw = true;
            run += number_after(t, "Tests run: ");
            failed += number_after(t, "Failures: ") + number_after(t, "Errors: ");
            skipped += number_after(t, "Skipped: ");
        } else if t.starts_with("[ERROR]") && t.contains("Test.") && t.contains(':') && !t.contains("Tests run") {
            failures.push(t.trim_start_matches("[ERROR]").trim().to_string());
        }
    }
    let ran = run.saturating_sub(skipped);
    let status = if !saw {
        Status::BuildError
    } else if failed > 0 {
        Status::Failing
    } else if !success {
        Status::BuildError
    } else if skipped > 0 && (ran == 0 || skipped * 2 >= run) {
        Status::NotStarted
    } else {
        Status::Passing
    };
    Outcome {
        status,
        passed: ran.saturating_sub(failed),
        failed,
        skipped,
        failures,
        raw: raw.to_string(),
    }
}

/// "... 5 passed; ..." -> 5 (the number directly before the marker)
fn number_before(s: &str, marker: &str) -> u32 {
    s.find(marker)
        .and_then(|i| s[..i].rsplit(|c: char| !c.is_ascii_digit()).next().and_then(|n| n.parse().ok()))
        .unwrap_or(0)
}

/// "Tests run: 5, ..." with marker "Tests run: " -> 5
fn number_after(s: &str, marker: &str) -> u32 {
    s.find(marker)
        .map(|i| s[i + marker.len()..].chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

fn report(p: &Problem, o: &RunOpts, r: &Outcome) {
    let what = format!(
        "{} [{}{}]",
        p.slug,
        o.lang.name(),
        if o.hidden { ", hidden" } else { "" }
    );
    match r.status {
        Status::BuildError => {
            println!("{} {}", red("BUILD FAILED"), bold(&what));
            let lines: Vec<&str> = r.raw.lines().collect();
            let start = lines.len().saturating_sub(60);
            for l in &lines[start..] {
                println!("{l}");
            }
        }
        Status::Failing => {
            println!(
                "{} {}  ({} passed, {} failed)",
                red("FAILING"),
                bold(&what),
                r.passed,
                r.failed
            );
            for f in &r.failures {
                println!("  {} {f}", red("x"));
            }
            if o.verbose {
                println!("\n{}", r.raw);
            } else {
                print_failure_details(&r.raw);
                println!("{}", dim("(use --verbose for the full output)"));
            }
        }
        Status::NotStarted => {
            println!(
                "{} {}  - the factory still returns nothing; {} tests skipped",
                yellow("NOT STARTED"),
                bold(&what),
                r.skipped.max(1)
            );
            println!(
                "{}",
                dim("  wire your design: rust -> create() in src/lib.rs, java -> solution/Solution.java")
            );
            println!("{}", dim("  add --strict to make 'not implemented' a failure"));
        }
        Status::Passing => {
            println!("{} {}  ({} tests)", green("PASSING"), bold(&what), r.passed);
            if !o.hidden {
                println!(
                    "{}",
                    dim(&format!(
                        "  next: `cargo play run {} --hidden --lang {}` for the interviewer scenarios",
                        p.number,
                        o.lang.name()
                    ))
                );
            }
        }
    }
}

fn print_failure_details(raw: &str) {
    let lines: Vec<&str> = raw.lines().collect();
    let mut seen: Vec<String> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim();
        let msg = if t.contains("panicked at") {
            // the message is on the following line(s) until a blank line or note
            let mut m = String::new();
            for next in lines.iter().skip(i + 1).take(4) {
                let n = next.trim();
                if n.is_empty() || n.starts_with("note:") || n.starts_with("error:") || n.starts_with("stack backtrace") {
                    break;
                }
                m.push_str(n);
                m.push(' ');
            }
            let name = t.split('\u{27}').nth(1).unwrap_or("test");
            format!("{name}: {}", m.trim())
        } else if t.starts_with("[ERROR]") && !t.contains("Tests run") && !t.contains("Failures:") {
            t.to_string()
        } else {
            continue;
        };
        if !seen.contains(&msg) {
            seen.push(msg);
        }
        if seen.len() >= 12 {
            break;
        }
    }
    for m in &seen {
        println!("    {m}");
    }
}

fn do_run(repo: &Repo, p: &Problem, o: &RunOpts) -> Result<Status, String> {
    println!("{}", dim(&format!("running {} tests for {} ...", o.lang.name(), p.slug)));
    let r = run_tests(repo, p, o)?;
    report(p, o, &r);
    if !o.hidden {
        repo.record(o.lang, p, r.status);
    }
    Ok(r.status)
}

// ---------- commands ----------

fn cmd_list(repo: &Repo, lang: Option<Lang>) {
    let metas = repo.metas();
    let progress = repo.load_progress();
    let langs: Vec<Lang> = lang.map(|l| vec![l]).unwrap_or_else(|| vec![Lang::Rust, Lang::Java]);
    print!(
        "{}",
        bold(&format!("{:>3}  {:<24} {:<12} {:>4} {:<3}", "#", "problem", "difficulty", "min", "lvl"))
    );
    for l in &langs {
        print!("  {}", bold(&format!("{:<13}", l.name())));
    }
    println!();
    let (mut done, mut total) = (0, 0);
    for p in repo.problems() {
        let m = metas.get(&p.number);
        print!(
            "{:>3}  {:<24} {:<12} {:>4} {:<3}",
            p.number,
            p.slug.split_once('-').map(|x| x.1).unwrap_or(&p.slug),
            m.map(|m| m.difficulty.as_str()).unwrap_or("?"),
            m.map(|m| m.minutes.as_str()).unwrap_or("?"),
            m.map(|m| m.level.as_str()).unwrap_or("?")
        );
        for l in &langs {
            let st = progress
                .get(&(l.name().to_string(), p.slug.clone()))
                .copied()
                .unwrap_or(Status::NotStarted);
            total += 1;
            if st == Status::Passing {
                done += 1;
            }
            let (label, width) = st.label();
            print!("  {label}{}", " ".repeat(13usize.saturating_sub(width)));
        }
        println!();
    }
    println!("\n{done}/{total} passing. Status comes from your last `cargo play run` / `watch`.");
}

fn cmd_next(repo: &Repo, lang: Lang) {
    let progress = repo.load_progress();
    let metas = repo.metas();
    for p in repo.problems() {
        let st = progress
            .get(&(lang.name().to_string(), p.slug.clone()))
            .copied()
            .unwrap_or(Status::NotStarted);
        if st != Status::Passing {
            let title = metas
                .get(&p.number)
                .map(|m| m.title.clone())
                .unwrap_or_else(|| p.slug.clone());
            println!("{} {} ({})", bold("Next:"), bold(&title), p.slug);
            println!("  read   {}", repo.problem_dir(lang, &p).join("README.md").display());
            println!("  work   cargo play watch {} --lang {}", p.number, lang.name());
            println!("  once   cargo play run {} --lang {}", p.number, lang.name());
            return;
        }
    }
    println!(
        "{}",
        green("Everything is passing. Try --hidden, the follow-ups, or the other language.")
    );
}

fn cmd_info(repo: &Repo, p: &Problem, lang: Lang, doc: &str, yes: bool) -> Result<(), String> {
    let file = match doc {
        "readme" | "problem" => "README.md",
        "interview" | "rules" => "INTERVIEW.md",
        "followups" | "follow-ups" => "FOLLOWUPS.md",
        "rubric" => "RUBRIC.md",
        "notes" => {
            if !yes {
                return Err(
                    "INTERVIEWER_NOTES.md contains hidden edge cases and traps; pass --yes if you really want it"
                        .into(),
                );
            }
            "INTERVIEWER_NOTES.md"
        }
        other => return Err(format!("unknown document '{other}' (readme|interview|followups|rubric|notes)")),
    };
    let path = repo.problem_dir(lang, p).join(file);
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("{}\n", dim(&path.display().to_string()));
    println!("{text}");
    Ok(())
}

fn watched_files(repo: &Repo, p: &Problem, lang: Lang) -> Vec<(PathBuf, SystemTime)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, SystemTime)>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if let Ok(m) = e.metadata().and_then(|m| m.modified()) {
                    out.push((path, m));
                }
            }
        }
    }
    let base = repo.problem_dir(lang, p);
    let roots: Vec<PathBuf> = match lang {
        Lang::Rust => vec![base.join("src"), base.join("tests")],
        Lang::Java => vec![base.join("src")],
    };
    let mut out = Vec::new();
    for r in roots {
        walk(&r, &mut out);
    }
    out.sort();
    out
}

fn cmd_watch(repo: &Repo, p: &Problem, o: &RunOpts) -> Result<(), String> {
    println!(
        "{}",
        bold(&format!(
            "Watching {} ({}) - edit your solution, results refresh on save. Ctrl+C to quit.",
            p.slug,
            o.lang.name()
        ))
    );
    let mut last = watched_files(repo, p, o.lang);
    loop {
        if color_on() {
            print!("\x1b[2J\x1b[H");
        }
        let status = do_run(repo, p, o)?;
        if status == Status::Passing {
            println!(
                "\n{}",
                green("Nice. Keep watching for refactors, or Ctrl+C and run `cargo play next`.")
            );
        }
        println!("{}", dim("\nwaiting for changes ..."));
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if watched_files(repo, p, o.lang) != last {
                std::thread::sleep(Duration::from_millis(300)); // let the editor finish writing
                last = watched_files(repo, p, o.lang);
                break;
            }
        }
    }
}

fn cmd_where(repo: &Repo, p: &Problem, lang: Lang) {
    let dir = repo.problem_dir(lang, p);
    println!("{}", dir.display());
    match lang {
        Lang::Rust => println!("  edit: {}", dir.join("src").join("lib.rs").display()),
        Lang::Java => println!(
            "  edit: {}",
            dir.join("src/main/java/solution/Solution.java").display()
        ),
    }
}

fn help() {
    println!(
        "{} - machine-coding / LLD playground runner

{}
  cargo play list [--lang rust|java]       all problems with your progress
  cargo play next [--lang L]               the first problem that is not passing yet
  cargo play run <problem> [options]       run ONE problem's tests
  cargo play watch <problem> [options]     re-run on every save (like rustlings watch)
  cargo play info <problem> [doc]          print readme | interview | followups | rubric
  cargo play where <problem> [--lang L]    path of the problem and the file to edit
  cargo play reset-progress                forget recorded status

{}
  <problem>          number (18) or unique part of the slug (lru, rate)
  --lang, -l L       rust (default, or $PLAYGROUND_LANG) | java
  --hidden           also run the interviewer scenarios
  --strict           treat 'not implemented' as a failure (STRICT=1)
  --verbose, -v      print the full test output on failure
  --yes              (info notes) confirm you want to read the interviewer notes

{}
  cargo play run 18
  cargo play watch rate -l java --strict
  cargo play info 19 followups",
        bold("play"),
        bold("Commands"),
        bold("Options"),
        bold("Examples")
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = real_main(args) {
        eprintln!("{} {e}", red("error:"));
        std::process::exit(2);
    }
}

fn real_main(args: Vec<String>) -> Result<(), String> {
    let mut lang = std::env::var("PLAYGROUND_LANG")
        .ok()
        .and_then(|s| Lang::parse(&s))
        .unwrap_or(Lang::Rust);
    let (mut lang_given, mut hidden, mut strict, mut verbose, mut yes) = (false, false, false, false, false);
    let mut positional: Vec<String> = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--lang" | "-l" => {
                let v = it.next().ok_or("--lang needs a value (rust|java)")?;
                lang = Lang::parse(&v).ok_or_else(|| format!("unknown language '{v}' (rust|java)"))?;
                lang_given = true;
            }
            "--hidden" => hidden = true,
            "--strict" => strict = true,
            "--verbose" | "-v" => verbose = true,
            "--yes" => yes = true,
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            _ => positional.push(a),
        }
    }
    let cmd = positional.first().cloned().unwrap_or_else(|| "help".to_string());
    if cmd == "help" {
        help();
        return Ok(());
    }
    let repo = Repo::locate()?;
    let opts = RunOpts {
        lang,
        hidden,
        strict,
        verbose,
    };
    let problem = || -> Result<Problem, String> {
        let q = positional.get(1).ok_or("which problem? e.g. `cargo play run 18`")?;
        repo.resolve(q)
    };
    match cmd.as_str() {
        "list" | "ls" => {
            cmd_list(&repo, if lang_given { Some(lang) } else { None });
            Ok(())
        }
        "next" => {
            cmd_next(&repo, lang);
            Ok(())
        }
        "run" | "test" => {
            let p = problem()?;
            if do_run(&repo, &p, &opts)? != Status::Passing {
                std::process::exit(1);
            }
            Ok(())
        }
        "watch" => cmd_watch(&repo, &problem()?, &opts),
        "info" => {
            let p = problem()?;
            let doc = positional.get(2).map(|s| s.as_str()).unwrap_or("readme");
            cmd_info(&repo, &p, lang, doc, yes)
        }
        "where" => {
            cmd_where(&repo, &problem()?, lang);
            Ok(())
        }
        "reset-progress" => {
            let _ = fs::remove_file(repo.progress_file());
            println!("progress cleared");
            Ok(())
        }
        other => Err(format!("unknown command '{other}'; try `cargo play help`")),
    }
}
