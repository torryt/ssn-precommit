use regex::Regex;
use std::collections::HashSet;

/// A match found in the diff.
pub struct SsnMatch {
    pub file: String,
    pub ssn: String,
}

/// Scans unified diff output for SSNs in added lines.
pub fn scan_diff(diff_output: &str, filename: &str, ignore: &HashSet<String>) -> Vec<SsnMatch> {
    // Match 11 consecutive digits, or 6 digits + space + 5 digits
    let ssn_re = Regex::new(r"\b(\d{11}|\d{6} \d{5})\b").expect("invalid regex");
    let mut matches = Vec::new();

    for line in diff_output.lines() {
        // Only look at added lines (start with '+', but not the +++ header)
        if !line.starts_with('+') || line.starts_with("+++") {
            continue;
        }

        for cap in ssn_re.find_iter(line) {
            // Normalize by stripping the space so ignore/masking works uniformly
            let ssn = cap.as_str().replace(' ', "");
            if !ignore.contains(&ssn) {
                matches.push(SsnMatch {
                    file: filename.to_string(),
                    ssn,
                });
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ignore() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn detects_11_digit_ssn_in_added_line() {
        let diff = "+Some text 12345678901 more text\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ssn, "12345678901");
    }

    #[test]
    fn detects_spaced_ssn_format() {
        let diff = "+ID: 123456 78901\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ssn, "12345678901"); // normalized
    }

    #[test]
    fn ignores_removed_lines() {
        let diff = "-removed 12345678901\n+clean line\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert!(matches.is_empty());
    }

    #[test]
    fn ignores_diff_header() {
        let diff = "+++ b/test.txt\n+12345678901\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert_eq!(matches.len(), 1); // only the second line
    }

    #[test]
    fn respects_ignore_set() {
        let mut ignore = HashSet::new();
        ignore.insert("12345678901".to_string());
        let diff = "+12345678901\n";
        let matches = scan_diff(diff, "test.txt", &ignore);
        assert!(matches.is_empty());
    }

    #[test]
    fn no_match_on_longer_numbers() {
        let diff = "+123456789012\n"; // 12 digits — word boundary prevents match
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert!(matches.is_empty());
    }

    #[test]
    fn multiple_ssns_on_one_line() {
        let diff = "+11111111111 22222222222\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn no_match_on_clean_content() {
        let diff = "+hello world\n+just some text 12345\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert!(matches.is_empty());
    }

    #[test]
    fn context_lines_are_skipped() {
        let diff = " context 12345678901\n+added 99887766554\n";
        let matches = scan_diff(diff, "test.txt", &empty_ignore());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].ssn, "99887766554");
    }
}
