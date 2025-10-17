use anyhow::{Context, Result};
use glob::{Pattern, PatternError};
use std::path::Path;

/// Handles include/exclude pattern matching for file watching
#[derive(Debug)]
pub struct PatternFilter {
    include_patterns: Vec<Pattern>,
    exclude_patterns: Vec<Pattern>,
}

/// Expand brace patterns like "*.{rs,toml}" into ["*.rs", "*.toml"]
fn expand_braces(pattern: &str) -> Vec<String> {
    // Look for pattern like "prefix{ext1,ext2,ext3}suffix"
    if let Some(start) = pattern.find('{')
        && let Some(end) = pattern.find('}')
        && start < end
    {
        let prefix = &pattern[..start];
        let suffix = &pattern[end + 1..];
        let extensions = &pattern[start + 1..end];

        return extensions
            .split(',')
            .map(|ext| format!("{}{}{}", prefix, ext.trim(), suffix))
            .collect();
    }

    // No braces found, return original pattern
    vec![pattern.to_string()]
}

impl PatternFilter {
    /// Create a new pattern filter with include and exclude patterns
    pub fn new(include_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Result<Self> {
        // Expand brace patterns before compilation
        let expanded_include: Vec<String> = include_patterns
            .iter()
            .flat_map(|p| {
                let expanded = expand_braces(p);
                if log::log_enabled!(log::Level::Debug) && expanded.len() > 1 {
                    log::debug!("Expanded include pattern '{}' to {:?}", p, expanded);
                }
                expanded
            })
            .collect();

        let expanded_exclude: Vec<String> = exclude_patterns
            .iter()
            .flat_map(|p| {
                let expanded = expand_braces(p);
                if log::log_enabled!(log::Level::Debug) && expanded.len() > 1 {
                    log::debug!("Expanded exclude pattern '{}' to {:?}", p, expanded);
                }
                expanded
            })
            .collect();

        let include_patterns = Self::compile_patterns(expanded_include)
            .context("Failed to compile include patterns")?;

        let exclude_patterns = Self::compile_patterns(expanded_exclude)
            .context("Failed to compile exclude patterns")?;

        Ok(Self {
            include_patterns,
            exclude_patterns,
        })
    }

    /// Check if a file path should be watched based on include/exclude patterns
    pub fn should_watch(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // If file matches any exclude pattern, don't watch it
        if self.matches_any_pattern(&self.exclude_patterns, &path_str) {
            log::debug!("File excluded by pattern: {}", path_str);
            return false;
        }

        // If there are include patterns, file must match at least one
        if !self.include_patterns.is_empty() {
            let matches = self.matches_any_pattern(&self.include_patterns, &path_str);
            if !matches {
                log::debug!("File doesn't match include patterns: {}", path_str);
            }
            return matches;
        }

        // If no include patterns specified, watch everything (that doesn't match exclude)
        true
    }

    /// Compile string patterns into glob Pattern objects
    fn compile_patterns(patterns: Vec<String>) -> Result<Vec<Pattern>, PatternError> {
        patterns.into_iter().map(|p| Pattern::new(&p)).collect()
    }

    /// Check if path matches any of the given patterns
    fn matches_any_pattern(&self, patterns: &[Pattern], path: &str) -> bool {
        patterns.iter().any(|pattern| {
            let matches = pattern.matches(path);
            if matches {
                log::debug!("Path '{}' matches pattern '{}'", path, pattern.as_str());
            }
            matches
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::PathBuf;

    #[test]
    fn test_include_patterns() {
        let filter =
            PatternFilter::new(vec!["*.rs".to_string(), "*.ts".to_string()], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(filter.should_watch(&PathBuf::from("app.ts")));
        assert!(!filter.should_watch(&PathBuf::from("readme.md")));
    }

    #[test]
    fn test_exclude_patterns() {
        let filter = PatternFilter::new(
            vec![],
            vec!["node_modules/**".to_string(), "*.tmp".to_string()],
        )
        .unwrap();

        assert!(!filter.should_watch(&PathBuf::from("node_modules/package/index.js")));
        assert!(!filter.should_watch(&PathBuf::from("temp.tmp")));
        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_include_and_exclude() {
        let filter =
            PatternFilter::new(vec!["*.rs".to_string()], vec!["target/**".to_string()]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("target/debug/main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("app.js")));
    }

    #[test]
    fn test_no_patterns_watches_all() {
        let filter = PatternFilter::new(vec![], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("any/file.txt")));
        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(filter.should_watch(&PathBuf::from("src/lib.rs")));
    }

    // Parameterized tests for include patterns
    #[rstest]
    #[case("*.md", "README.md", true)]
    #[case("*.md", "docs/guide.md", true)]
    #[case("*.md", "main.rs", false)]
    #[case("**/*.rs", "src/main.rs", true)]
    #[case("**/*.rs", "tests/integration.rs", true)]
    #[case("**/*.rs", "README.md", false)]
    #[case("src/**/*.rs", "src/main.rs", true)]
    #[case("src/**/*.rs", "src/lib/util.rs", true)]
    #[case("src/**/*.rs", "tests/test.rs", false)]
    fn test_include_pattern_matching(
        #[case] pattern: &str,
        #[case] path: &str,
        #[case] should_match: bool,
    ) {
        let filter = PatternFilter::new(vec![pattern.to_string()], vec![]).unwrap();
        assert_eq!(
            should_match,
            filter.should_watch(&PathBuf::from(path)),
            "Pattern '{}' with path '{}' should be {}",
            pattern,
            path,
            if should_match { "matched" } else { "rejected" }
        );
    }

    // Parameterized tests for exclude patterns
    #[rstest]
    #[case("*.log", "app.log", false)]
    #[case("*.log", "debug.log", false)]
    #[case("*.log", "src/main.rs", true)]
    #[case("**/node_modules/**", "node_modules/pkg/index.js", false)]
    #[case("**/.git/**", ".git/config", false)]
    #[case("**/target/**", "target/release/app", false)]
    #[case("**/target/**", "src/main.rs", true)]
    fn test_exclude_pattern_matching(
        #[case] pattern: &str,
        #[case] path: &str,
        #[case] should_watch: bool,
    ) {
        let filter = PatternFilter::new(vec![], vec![pattern.to_string()]).unwrap();
        assert_eq!(
            should_watch,
            filter.should_watch(&PathBuf::from(path)),
            "Exclude pattern '{}' with path '{}' should be {}",
            pattern,
            path,
            if should_watch { "allowed" } else { "blocked" }
        );
    }

    #[test]
    fn test_multiple_wildcards() {
        let filter = PatternFilter::new(
            vec!["**/*.rs".to_string(), "**/*.toml".to_string()],
            vec!["**/target/**".to_string()],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
        assert!(filter.should_watch(&PathBuf::from("Cargo.toml")));
        assert!(!filter.should_watch(&PathBuf::from("target/debug/main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("README.md")));
    }

    #[test]
    fn test_exclude_takes_precedence() {
        let filter = PatternFilter::new(
            vec!["**/*.rs".to_string()],
            vec!["**/test_*.rs".to_string()],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("src/test_helper.rs")));
        assert!(!filter.should_watch(&PathBuf::from("tests/test_integration.rs")));
    }

    #[test]
    fn test_invalid_include_pattern_returns_error() {
        let result = PatternFilter::new(vec!["[invalid".to_string()], vec![]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to compile include patterns"));
    }

    #[test]
    fn test_invalid_exclude_pattern_returns_error() {
        let result = PatternFilter::new(vec![], vec!["[invalid".to_string()]);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to compile exclude patterns"));
    }

    #[test]
    fn test_complex_glob_patterns() {
        let filter = PatternFilter::new(
            vec![
                "src/**/*.rs".to_string(),
                "src/**/*.toml".to_string(),
                "src/**/*.md".to_string(),
            ],
            vec![],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/lib.rs")));
        assert!(filter.should_watch(&PathBuf::from("src/bin/main.rs")));
        assert!(filter.should_watch(&PathBuf::from("src/README.md")));
        assert!(!filter.should_watch(&PathBuf::from("tests/test.rs")));
    }

    // Removed: test_nested_exclude_patterns - now covered by parameterized test_exclude_pattern_matching

    #[test]
    fn test_case_sensitive_patterns() {
        let filter = PatternFilter::new(vec!["*.RS".to_string()], vec![]).unwrap();

        // Glob patterns are case-sensitive by default
        assert!(filter.should_watch(&PathBuf::from("MAIN.RS")));
        assert!(!filter.should_watch(&PathBuf::from("main.rs")));
    }

    #[test]
    fn test_exact_path_match() {
        let filter = PatternFilter::new(vec!["Cargo.toml".to_string()], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("Cargo.toml")));
        assert!(!filter.should_watch(&PathBuf::from("src/Cargo.toml")));
    }

    #[rstest]
    #[case("test?.rs", "test1.rs", true)]
    #[case("test?.rs", "testa.rs", true)]
    #[case("test?.rs", "test12.rs", false)]
    #[case("test?.rs", "test.rs", false)]
    #[case("*", "file with spaces.txt", true)]
    #[case("*", "path/with spaces/file.rs", true)]
    #[case("*.txt", "日本語.txt", true)]
    #[case("*.txt", "émoji🎉.txt", true)]
    #[case("*.txt", "日本語.rs", false)]
    fn test_special_patterns_and_edge_cases(
        #[case] pattern: &str,
        #[case] path: &str,
        #[case] should_match: bool,
    ) {
        let filter = PatternFilter::new(vec![pattern.to_string()], vec![]).unwrap();
        assert_eq!(
            should_match,
            filter.should_watch(&PathBuf::from(path)),
            "Pattern '{}' with path '{}' should be {}",
            pattern,
            path,
            if should_match { "matched" } else { "rejected" }
        );
    }

    #[test]
    fn test_empty_include_with_excludes() {
        let filter =
            PatternFilter::new(vec![], vec!["*.tmp".to_string(), "*.bak".to_string()]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("file.tmp")));
        assert!(!filter.should_watch(&PathBuf::from("backup.bak")));
    }

    #[test]
    fn test_multiple_include_patterns() {
        let filter = PatternFilter::new(
            vec!["*.rs".to_string(), "*.toml".to_string(), "*.md".to_string()],
            vec![],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(filter.should_watch(&PathBuf::from("Cargo.toml")));
        assert!(filter.should_watch(&PathBuf::from("README.md")));
        assert!(!filter.should_watch(&PathBuf::from("script.sh")));
    }

    #[test]
    fn test_pattern_with_directory_separator() {
        let filter = PatternFilter::new(vec!["src/*.rs".to_string()], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
        assert!(filter.should_watch(&PathBuf::from("src/lib.rs")));
        assert!(!filter.should_watch(&PathBuf::from("tests/test.rs")));
        assert!(!filter.should_watch(&PathBuf::from("main.rs")));
    }

    // Brace expansion tests
    #[test]
    fn test_brace_expansion_basic() {
        let expanded = expand_braces("*.{rs,toml}");
        assert_eq!(expanded, vec!["*.rs", "*.toml"]);
    }

    #[test]
    fn test_brace_expansion_three_extensions() {
        let expanded = expand_braces("*.{js,ts,jsx}");
        assert_eq!(expanded, vec!["*.js", "*.ts", "*.jsx"]);
    }

    #[test]
    fn test_brace_expansion_with_prefix_and_suffix() {
        let expanded = expand_braces("src/**/*.{rs,toml}");
        assert_eq!(expanded, vec!["src/**/*.rs", "src/**/*.toml"]);
    }

    #[test]
    fn test_brace_expansion_with_spaces() {
        let expanded = expand_braces("*.{rs, toml, md}");
        assert_eq!(expanded, vec!["*.rs", "*.toml", "*.md"]);
    }

    #[test]
    fn test_brace_expansion_no_braces() {
        let expanded = expand_braces("*.rs");
        assert_eq!(expanded, vec!["*.rs"]);
    }

    #[test]
    fn test_brace_expansion_malformed_no_closing() {
        let expanded = expand_braces("*.{rs,toml");
        assert_eq!(expanded, vec!["*.{rs,toml"]);
    }

    #[test]
    fn test_brace_expansion_malformed_no_opening() {
        let expanded = expand_braces("*.rs,toml}");
        assert_eq!(expanded, vec!["*.rs,toml}"]);
    }

    #[test]
    fn test_brace_expansion_empty() {
        let expanded = expand_braces("*.{}");
        assert_eq!(expanded, vec!["*."]);
    }

    #[test]
    fn test_filter_with_brace_expansion() {
        let filter = PatternFilter::new(vec!["*.{rs,toml}".to_string()], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(filter.should_watch(&PathBuf::from("Cargo.toml")));
        assert!(!filter.should_watch(&PathBuf::from("README.md")));
    }

    #[test]
    fn test_filter_with_brace_expansion_typescript() {
        let filter = PatternFilter::new(vec!["*.{ts,tsx}".to_string()], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("App.tsx")));
        assert!(filter.should_watch(&PathBuf::from("utils.ts")));
        assert!(!filter.should_watch(&PathBuf::from("index.js")));
    }

    #[test]
    fn test_filter_with_multiple_brace_patterns() {
        let filter = PatternFilter::new(
            vec!["*.{rs,toml}".to_string(), "*.{md,txt}".to_string()],
            vec![],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(filter.should_watch(&PathBuf::from("Cargo.toml")));
        assert!(filter.should_watch(&PathBuf::from("README.md")));
        assert!(filter.should_watch(&PathBuf::from("notes.txt")));
        assert!(!filter.should_watch(&PathBuf::from("script.sh")));
    }

    #[test]
    fn test_filter_brace_expansion_with_exclude() {
        let filter = PatternFilter::new(
            vec!["*.{rs,toml}".to_string()],
            vec!["target/**".to_string()],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
        assert!(filter.should_watch(&PathBuf::from("Cargo.toml")));
        assert!(!filter.should_watch(&PathBuf::from("target/debug/main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("README.md")));
    }

    #[rstest]
    #[case("*.{ts,tsx}", "file.ts", true)]
    #[case("*.{ts,tsx}", "file.tsx", true)]
    #[case("*.{ts,tsx}", "file.js", false)]
    #[case("*.{js,jsx,ts,tsx}", "app.js", true)]
    #[case("*.{js,jsx,ts,tsx}", "app.jsx", true)]
    #[case("*.{js,jsx,ts,tsx}", "app.ts", true)]
    #[case("*.{js,jsx,ts,tsx}", "app.tsx", true)]
    #[case("*.{js,jsx,ts,tsx}", "app.py", false)]
    #[case("src/**/*.{rs,toml}", "src/main.rs", true)]
    #[case("src/**/*.{rs,toml}", "src/lib/util.rs", true)]
    #[case("src/**/*.{rs,toml}", "src/Cargo.toml", true)]
    #[case("src/**/*.{rs,toml}", "tests/test.rs", false)]
    fn test_brace_pattern_matching(
        #[case] pattern: &str,
        #[case] path: &str,
        #[case] should_match: bool,
    ) {
        let filter = PatternFilter::new(vec![pattern.to_string()], vec![]).unwrap();
        assert_eq!(
            should_match,
            filter.should_watch(&PathBuf::from(path)),
            "Brace pattern '{}' with path '{}' should be {}",
            pattern,
            path,
            if should_match { "matched" } else { "rejected" }
        );
    }

    #[test]
    fn test_double_star_pattern() {
        let filter = PatternFilter::new(vec!["**/test/**/*.rs".to_string()], vec![]).unwrap();

        assert!(filter.should_watch(&PathBuf::from("test/unit/test.rs")));
        assert!(filter.should_watch(&PathBuf::from("src/test/integration/test.rs")));
        assert!(!filter.should_watch(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_pattern_filter_debug() {
        let filter = PatternFilter::new(vec!["*.rs".to_string()], vec![]).unwrap();
        let debug_str = format!("{:?}", filter);
        assert!(debug_str.contains("PatternFilter"));
    }

    #[test]
    fn test_multiple_invalid_patterns() {
        let result = PatternFilter::new(
            vec!["[invalid".to_string(), "[also_invalid".to_string()],
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_overlapping_patterns() {
        let filter =
            PatternFilter::new(vec!["*.rs".to_string(), "src/*.rs".to_string()], vec![]).unwrap();

        // Both patterns match, should still work
        assert!(filter.should_watch(&PathBuf::from("main.rs")));
        assert!(filter.should_watch(&PathBuf::from("src/lib.rs")));
    }

    #[test]
    fn test_exclude_overrides_overlapping_include() {
        let filter = PatternFilter::new(
            vec!["**/*.rs".to_string()],
            vec!["**/test_*.rs".to_string()],
        )
        .unwrap();

        assert!(filter.should_watch(&PathBuf::from("src/main.rs")));
        assert!(!filter.should_watch(&PathBuf::from("src/test_helper.rs")));
    }
}
