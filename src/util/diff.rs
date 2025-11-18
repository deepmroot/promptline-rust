//! Diff generation utilities

use colored::Colorize;
use similar::{ChangeTag, TextDiff};

/// Generate a unified diff between two texts
pub fn generate_diff(original: &str, modified: &str) -> String {
    let diff = TextDiff::from_lines(original, modified);

    let mut output = Vec::new();

    for change in diff.iter_all_changes() {
        let line = format!("{}", change);
        let formatted = match change.tag() {
            ChangeTag::Delete => format!("- {}", line).red().to_string(),
            ChangeTag::Insert => format!("+ {}", line).green().to_string(),
            ChangeTag::Equal => format!("  {}", line),
        };
        output.push(formatted);
    }

    output.join("")
}

/// Generate a colored diff for terminal display
pub fn display_diff(path: &str, original: &str, modified: &str) {
    println!("\n{}", "=".repeat(60));
    println!("📝 File: {}", path.bold());
    println!("{}", "=".repeat(60));

    let diff = generate_diff(original, modified);
    println!("{}", diff);

    println!("{}", "=".repeat(60));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_diff() {
        let original = "line 1\nline 2\nline 3\n";
        let modified = "line 1\nline 2 modified\nline 3\n";

        let diff = generate_diff(original, modified);

        assert!(diff.contains("line 1"));
        assert!(diff.contains("line 2"));
        assert!(diff.contains("modified"));
    }

    #[test]
    fn test_empty_diff() {
        let text = "same text\n";
        let diff = generate_diff(text, text);

        assert!(diff.contains("same text"));
        assert!(!diff.contains('+'));
        assert!(!diff.contains('-'));
    }
}
