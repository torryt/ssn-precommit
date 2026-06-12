use regex::Regex;
use std::process::{Command, ExitCode};

/// A match found in the diff.
struct SsnMatch {
    file: String,
    ssn: String,
}

fn main() -> ExitCode {
    let files: Vec<String> = std::env::args().skip(1).collect();

    if files.is_empty() {
        return ExitCode::SUCCESS;
    }

    let matches = find_ssns_in_staged_diff(&files);

    if matches.is_empty() {
        return ExitCode::SUCCESS;
    }

    print_warning(&matches);
    ExitCode::FAILURE
}

/// Runs `git diff --cached` for the given files and scans added lines for SSNs.
fn find_ssns_in_staged_diff(files: &[String]) -> Vec<SsnMatch> {
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
                matches.push(SsnMatch {
                    file: file.clone(),
                    ssn: cap.as_str().to_string(),
                });
            }
        }
    }

    matches
}

fn print_warning(matches: &[SsnMatch]) {
    eprintln!();
    eprintln!("============================================================");
    eprintln!("  WARNING: Potential SSN(s) detected in staged changes!");
    eprintln!("============================================================");
    eprintln!();
    eprintln!("  Found {} potential 11-digit SSN(s):", matches.len());
    eprintln!();

    for m in matches {
        let masked = format!("{}*****", &m.ssn[..6]);
        eprintln!("    {}: {} (full: {})", m.file, masked, m.ssn);
    }

    eprintln!();
    eprintln!("============================================================");
    eprintln!();
    eprintln!("  Commit blocked. If these are not real SSNs, bypass with:");
    eprintln!("    git commit --no-verify");
    eprintln!();
}
