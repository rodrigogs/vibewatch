use clap::Parser;
use std::path::PathBuf;

mod filter;
mod watcher;

// Help section headings
const FILTERING_HELP: &str = "File Filtering";
const COMMANDS_HELP: &str = "Command Execution";
const GENERAL_HELP: &str = "General Options";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "vibewatch")]
#[command(
    about = "A powerful file watcher with command execution",
    long_about = "vibewatch watches a directory for file changes and executes commands when events occur.\n\nIt supports glob patterns for precise filtering and template substitution for command execution.\nInspired by tools like watchexec, entr, and nodemon, but with a focus on simplicity and reliability.",
    after_help = "EXAMPLES:\n\n  # Watch current directory and run tests on any change\n  vibewatch . --on-change 'npm test'\n\n  # Watch Rust files and format them when modified\n  vibewatch src --include '*.rs' --on-modify 'rustfmt {file_path}'\n\n  # Watch TypeScript files, exclude node_modules, run linter\n  vibewatch . --include '*.{ts,tsx}' --exclude 'node_modules/**' --on-modify 'npx eslint {file_path} --fix'\n\n  # Different commands for different events\n  vibewatch src --on-create 'git add {file_path}' --on-modify 'cargo check' --on-delete 'echo Removed: {relative_path}'\n\n  # Watch docs and rebuild on changes\n  vibewatch docs --include '*.md' --on-change 'mdbook build'\n\nTEMPLATES:\n  {file_path}      - Full path to the changed file\n  {relative_path}  - Path relative to watched directory\n  {absolute_path}  - Absolute path to the changed file\n  {event_type}     - Type of event (create, modify, delete)\n\nNOTE:\n  Commands are executed asynchronously. Multiple events may trigger\n  overlapping command executions."
)]
struct Args {
    /// Root directory to watch for file changes (recursively)
    #[arg(value_name = "DIRECTORY")]
    #[arg(
        help = "Path to directory to monitor. Can be relative (e.g., '.', 'src') or absolute. Watches all subdirectories recursively"
    )]
    directory: PathBuf,

    /// Exclude patterns (glob patterns to ignore)
    #[arg(short, long, value_name = "PATTERN", help_heading = FILTERING_HELP)]
    #[arg(
        help = "Exclude files/directories matching these glob patterns\n\nExamples: 'node_modules/**', '.git/**', 'target/**', '*.tmp'\nCan be used multiple times to exclude different patterns"
    )]
    exclude: Vec<String>,

    /// Include patterns (glob patterns to watch)
    #[arg(short, long, value_name = "PATTERN", help_heading = FILTERING_HELP)]
    #[arg(
        help = "Only watch files matching these glob patterns\n\nExamples: '*.rs', '**/*.js', 'src/**/*.{ts,tsx}', '*.{md,txt}'\nIf not specified, watches all files. Can be used multiple times"
    )]
    include: Vec<String>,

    /// Enable verbose logging output
    #[arg(short, long, help_heading = GENERAL_HELP)]
    #[arg(
        help = "Show detailed debug information about file events, pattern matching, and command execution"
    )]
    verbose: bool,

    /// Command to execute when files are created
    #[arg(long, value_name = "COMMAND", help_heading = COMMANDS_HELP)]
    #[arg(
        help = "Run this command when NEW files are created\n\nTemplates: {file_path}, {relative_path}, {absolute_path}, {event_type}\nExample: --on-create 'git add {file_path}'"
    )]
    on_create: Option<String>,

    /// Command to execute when files are modified
    #[arg(long, value_name = "COMMAND", help_heading = COMMANDS_HELP)]
    #[arg(
        help = "Run this command when EXISTING files are modified/updated\n\nTemplates: {file_path}, {relative_path}, {absolute_path}, {event_type}\nExample: --on-modify 'npx eslint {file_path} --fix'"
    )]
    on_modify: Option<String>,

    /// Command to execute when files are deleted
    #[arg(long, value_name = "COMMAND", help_heading = COMMANDS_HELP)]
    #[arg(
        help = "Run this command when files are DELETED/removed\n\nTemplates: {file_path}, {relative_path}, {absolute_path}, {event_type}\nExample: --on-delete 'echo File {relative_path} was removed'"
    )]
    on_delete: Option<String>,

    /// Command to execute on ANY file change (fallback for all events)
    #[arg(long, value_name = "COMMAND", help_heading = COMMANDS_HELP)]
    #[arg(
        help = "Run this command for ANY file event (create/modify/delete)\n\nActs as fallback when specific --on-* commands are not set\nTemplates: {file_path}, {relative_path}, {absolute_path}, {event_type}\nExample: --on-change 'echo {event_type}: {relative_path}'"
    )]
    on_change: Option<String>,
}

// Separate function for testability
fn create_watcher_from_args(args: Args) -> anyhow::Result<watcher::FileWatcher> {
    watcher::FileWatcher::new(
        args.directory,
        args.include,
        args.exclude,
        watcher::CommandConfig {
            on_create: args.on_create,
            on_modify: args.on_modify,
            on_delete: args.on_delete,
            on_change: args.on_change,
        },
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logger
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    log::info!("Starting vibewatch file watcher");
    log::info!("Watching directory: {}", args.directory.display());

    if !args.exclude.is_empty() {
        log::info!("Exclude patterns: {:?}", args.exclude);
    }

    if !args.include.is_empty() {
        log::info!("Include patterns: {:?}", args.include);
    }

    // Create and start the file watcher
    let mut watcher = create_watcher_from_args(args)?;
    watcher.start_watching()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use rstest::rstest;

    #[test]
    fn test_cli_verify_app() {
        // Verify the CLI app is valid
        Args::command().debug_assert();
    }

    #[test]
    fn test_args_basic_directory() {
        let args = Args::parse_from(["vibewatch", "."]);
        assert_eq!(args.directory, PathBuf::from("."));
        assert!(args.exclude.is_empty());
        assert!(args.include.is_empty());
        assert!(!args.verbose);
    }

    #[test]
    fn test_args_with_include_patterns() {
        let args = Args::parse_from(["vibewatch", ".", "--include", "*.rs", "--include", "*.toml"]);
        assert_eq!(args.directory, PathBuf::from("."));
        assert_eq!(args.include, vec!["*.rs", "*.toml"]);
    }

    #[test]
    fn test_args_with_exclude_patterns() {
        let args = Args::parse_from([
            "vibewatch",
            ".",
            "--exclude",
            "target/**",
            "--exclude",
            "node_modules/**",
        ]);
        assert_eq!(args.exclude, vec!["target/**", "node_modules/**"]);
    }

    #[test]
    fn test_args_with_verbose() {
        let args = Args::parse_from(["vibewatch", ".", "--verbose"]);
        assert!(args.verbose);
    }

    // Parameterized tests for command flags
    #[rstest]
    #[case("--on-create", "echo created", "on_create")]
    #[case("--on-modify", "echo modified", "on_modify")]
    #[case("--on-delete", "echo deleted", "on_delete")]
    #[case("--on-change", "echo changed", "on_change")]
    fn test_args_command_flags(
        #[case] flag: &str,
        #[case] command: &str,
        #[case] field_name: &str,
    ) {
        let args = Args::parse_from(["vibewatch", ".", flag, command]);
        let expected = Some(command.to_string());

        let actual = match field_name {
            "on_create" => &args.on_create,
            "on_modify" => &args.on_modify,
            "on_delete" => &args.on_delete,
            "on_change" => &args.on_change,
            _ => panic!("Unknown field: {}", field_name),
        };

        assert_eq!(
            actual, &expected,
            "Flag {} with command '{}' should be parsed correctly",
            flag, command
        );
    }

    #[test]
    fn test_args_all_options_combined() {
        let args = Args::parse_from([
            "vibewatch",
            "/tmp/watch",
            "--include",
            "*.rs",
            "--exclude",
            "target/**",
            "--verbose",
            "--on-create",
            "git add {file_path}",
            "--on-modify",
            "cargo check",
            "--on-delete",
            "echo removed",
            "--on-change",
            "echo changed",
        ]);

        assert_eq!(args.directory, PathBuf::from("/tmp/watch"));
        assert_eq!(args.include, vec!["*.rs"]);
        assert_eq!(args.exclude, vec!["target/**"]);
        assert!(args.verbose);
        assert_eq!(args.on_create, Some("git add {file_path}".to_string()));
        assert_eq!(args.on_modify, Some("cargo check".to_string()));
        assert_eq!(args.on_delete, Some("echo removed".to_string()));
        assert_eq!(args.on_change, Some("echo changed".to_string()));
    }

    #[test]
    fn test_args_short_flags() {
        let args = Args::parse_from(["vibewatch", ".", "-i", "*.rs", "-e", "target/**", "-v"]);

        assert_eq!(args.include, vec!["*.rs"]);
        assert_eq!(args.exclude, vec!["target/**"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_args_multiple_include_exclude() {
        let args = Args::parse_from([
            "vibewatch",
            ".",
            "-i",
            "*.rs",
            "-i",
            "*.toml",
            "-i",
            "*.md",
            "-e",
            "target/**",
            "-e",
            ".git/**",
            "-e",
            "node_modules/**",
        ]);

        assert_eq!(args.include, vec!["*.rs", "*.toml", "*.md"]);
        assert_eq!(
            args.exclude,
            vec!["target/**", ".git/**", "node_modules/**"]
        );
    }

    #[rstest]
    #[case("src", "src")]
    #[case("/tmp/test", "/tmp/test")]
    #[case(".", ".")]
    #[case("./target", "./target")]
    fn test_args_directory_paths(#[case] path: &str, #[case] expected: &str) {
        let args = Args::parse_from(["vibewatch", path]);
        assert_eq!(
            args.directory,
            PathBuf::from(expected),
            "Directory path '{}' should be parsed correctly",
            path
        );
    }

    #[test]
    fn test_args_no_commands() {
        let args = Args::parse_from(["vibewatch", "."]);
        assert_eq!(args.on_create, None);
        assert_eq!(args.on_modify, None);
        assert_eq!(args.on_delete, None);
        assert_eq!(args.on_change, None);
    }

    #[test]
    fn test_args_command_with_template_variables() {
        let args = Args::parse_from([
            "vibewatch",
            ".",
            "--on-modify",
            "echo {event_type}: {relative_path}",
        ]);
        assert_eq!(
            args.on_modify,
            Some("echo {event_type}: {relative_path}".to_string())
        );
    }

    #[test]
    fn test_args_command_with_quotes() {
        let args = Args::parse_from([
            "vibewatch",
            ".",
            "--on-change",
            "echo 'File changed: {file_path}'",
        ]);
        assert_eq!(
            args.on_change,
            Some("echo 'File changed: {file_path}'".to_string())
        );
    }

    #[test]
    fn test_args_minimal() {
        let args = Args::parse_from(["vibewatch", "."]);
        assert_eq!(args.directory, PathBuf::from("."));
        assert!(args.include.is_empty());
        assert!(args.exclude.is_empty());
        assert!(!args.verbose);
        assert!(args.on_create.is_none());
        assert!(args.on_modify.is_none());
        assert!(args.on_delete.is_none());
        assert!(args.on_change.is_none());
    }

    #[test]
    fn test_create_watcher_from_args_valid() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let args = Args {
            directory: temp_dir.path().to_path_buf(),
            exclude: vec![],
            include: vec![],
            verbose: false,
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = create_watcher_from_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_watcher_from_args_with_patterns() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let args = Args {
            directory: temp_dir.path().to_path_buf(),
            exclude: vec!["*.tmp".to_string()],
            include: vec!["*.rs".to_string()],
            verbose: true,
            on_create: Some("echo created".to_string()),
            on_modify: Some("echo modified".to_string()),
            on_delete: Some("echo deleted".to_string()),
            on_change: Some("echo changed".to_string()),
        };

        let result = create_watcher_from_args(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_watcher_from_args_invalid_directory() {
        let args = Args {
            directory: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            exclude: vec![],
            include: vec![],
            verbose: false,
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = create_watcher_from_args(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_watcher_from_args_invalid_patterns() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let args = Args {
            directory: temp_dir.path().to_path_buf(),
            exclude: vec![],
            include: vec!["[invalid".to_string()],
            verbose: false,
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = create_watcher_from_args(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_logger_initialization_verbose() {
        // Test that logger initialization doesn't panic with verbose mode
        // We use try_init to avoid conflicts with other tests
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        // If we get here, initialization succeeded (test passes)
    }

    #[test]
    fn test_logger_initialization_normal() {
        // Test that logger initialization doesn't panic with normal mode
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        // If we get here, initialization succeeded (test passes)
    }

    #[test]
    fn test_log_statements_coverage() {
        use tempfile::TempDir;

        // This test ensures log statements are executed
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        let temp_dir = TempDir::new().unwrap();

        // Simulate what main() does with logging
        log::info!("Starting vibewatch file watcher");
        log::info!("Watching directory: {}", temp_dir.path().display());

        let exclude_patterns = vec!["*.tmp".to_string()];
        if !exclude_patterns.is_empty() {
            log::info!("Exclude patterns: {:?}", exclude_patterns);
        }

        let include_patterns = vec!["*.rs".to_string()];
        if !include_patterns.is_empty() {
            log::info!("Include patterns: {:?}", include_patterns);
        }

        // Test passes if we reach here without panicking
    }
}
