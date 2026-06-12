use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::process::{Command, ExitCode};

/// A match found in the diff.
struct SsnMatch {
    file: String,
    ssn: String,
}

/// Parsed CLI options.
struct Options {
    mask: bool,
    ignore: HashSet<String>,
    files: Vec<String>,
}

fn parse_args() -> Options {
    let mut mask = true;
    let mut ignore = HashSet::new();
    let mut files = Vec::new();
    let mut args = std::env::args().skip(1).peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-mask" => mask = false,
            "--mask" => mask = true,
            "--ignore" => {
                if let Some(val) = args.next() {
                    for s in val.split(',') {
                        let trimmed = s.trim();
                        if !trimmed.is_empty() {
                            ignore.insert(trimmed.to_string());
                        }
                    }
                }
            }
            a if a.starts_with("--ignore=") => {
                let val = &a["--ignore=".len()..];
                for s in val.split(',') {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        ignore.insert(trimmed.to_string());
                    }
                }
            }
            _ => files.push(arg),
        }
    }

    // Also read from .ssnignore if it exists
    if let Ok(contents) = fs::read_to_string(".ssnignore") {
        for line in contents.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                ignore.insert(trimmed.to_string());
            }
        }
    }

    Options {
        mask,
        ignore,
        files,
    }
}

fn main() -> ExitCode {
    let opts = parse_args();

    if opts.files.is_empty() {
        return ExitCode::SUCCESS;
    }

    let matches = find_ssns_in_staged_diff(&opts.files, &opts.ignore);

    if matches.is_empty() {
        return ExitCode::SUCCESS;
    }

    print_warning(&matches, opts.mask);
    ExitCode::FAILURE
}

/// Runs `git diff --cached` for the given files and scans added lines for SSNs.
fn find_ssns_in_staged_diff(files: &[String], ignore: &HashSet<String>) -> Vec<SsnMatch> {
    let ssn_re = Regex::new(r"\b\d{11}\b").expect("invalid regex");
    let mut matches = Vec::new();

    for file in files {
        let output = Command::new("git")
            .args(["diff", "--cached", "-U0", "--", file])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => continue,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            // Only look at added lines (start with '+', but not the +++ header)
            if !line.starts_with('+') || line.starts_with("+++") {
                continue;
            }

            for cap in ssn_re.find_iter(line) {
                let ssn = cap.as_str().to_string();
                if !ignore.contains(&ssn) {
                    matches.push(SsnMatch {
                        file: file.clone(),
                        ssn,
                    });
                }
            }
        }
    }

    matches
}

fn print_warning(matches: &[SsnMatch], mask: bool) {
    eprintln!();
    eprintln!("============================================================");
    eprintln!("  WARNING: Potential SSN(s) detected in staged changes!");
    eprintln!("============================================================");
    eprintln!();
    eprintln!("  Found {} potential 11-digit SSN(s):", matches.len());
    eprintln!();

    for m in matches {
        if mask {
            let masked = format!("{}*****", &m.ssn[..6]);
            eprintln!("    {}: {}", m.file, masked);
        } else {
            eprintln!("    {}: {}", m.file, m.ssn);
        }
    }

    eprintln!();
    eprintln!("============================================================");
    eprintln!();
    eprintln!("  Commit blocked. If these are not real SSNs, bypass with:");
    eprintln!("    git commit --no-verify");
    eprintln!();
    eprintln!("  To permanently ignore a number, add it to .ssnignore or");
    eprintln!("  pass --ignore in your .pre-commit-config.yaml args.");
    eprintln!();
}
