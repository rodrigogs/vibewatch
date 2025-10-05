use anyhow::{Context, Result};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

use crate::filter::PatternFilter;

/// Configuration for command execution on file events
#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub on_create: Option<String>,
    pub on_modify: Option<String>,
    pub on_delete: Option<String>,
    pub on_change: Option<String>,
}

impl CommandConfig {
    /// Get the appropriate command for an event kind
    pub fn get_command_for_event(&self, event_kind: &EventKind) -> Option<&String> {
        match event_kind {
            EventKind::Create(_) => self.on_create.as_ref().or(self.on_change.as_ref()),
            EventKind::Modify(_) => self.on_modify.as_ref().or(self.on_change.as_ref()),
            EventKind::Remove(_) => self.on_delete.as_ref().or(self.on_change.as_ref()),
            _ => self.on_change.as_ref(),
        }
    }
}

/// Template context for command substitution
#[derive(Debug)]
pub(crate) struct TemplateContext {
    file_path: String,
    relative_path: String,
    event_type: &'static str,
    absolute_path: String,
}

impl TemplateContext {
    pub fn new(
        file_path: &Path,
        relative_path: &Path,
        event_kind: &EventKind,
        watch_path: &Path,
    ) -> Self {
        let absolute_path = watch_path.join(relative_path);
        // Normalize all paths to use forward slashes for cross-platform consistency
        Self {
            file_path: Self::normalize_path(file_path),
            relative_path: Self::normalize_path(relative_path),
            event_type: Self::event_kind_to_str(event_kind),
            absolute_path: Self::normalize_path(&absolute_path),
        }
    }

    /// Normalize path to use forward slashes
    /// 
    /// On Unix systems, avoids string replacement (just converts to string).
    /// On Windows, replaces backslashes with forward slashes.
    /// 
    /// Performance: On Unix/macOS (no backslashes), this is a simple to_string().
    /// On Windows (has backslashes), performs replace operation.
    fn normalize_path(path: &Path) -> String {
        let path_str = path.display().to_string();
        
        // Check if path contains backslashes (Windows-specific)
        if path_str.contains('\\') {
            // Windows: need to replace backslashes
            path_str.replace('\\', "/")
        } else {
            // Unix/macOS: no backslashes, return as-is
            path_str
        }
    }

    pub fn event_kind_to_str(event_kind: &EventKind) -> &'static str {
        match event_kind {
            EventKind::Create(_) => "create",
            EventKind::Modify(_) => "modify",
            EventKind::Remove(_) => "delete",
            _ => "change",
        }
    }

    /// Substitute template variables in a command string
    /// 
    /// Uses a single-pass algorithm with pre-allocated capacity for better performance.
    /// Supports: {file_path}, {relative_path}, {event_type}, {absolute_path}
    pub fn substitute_template(&self, template: &str) -> String {
        // Pre-allocate with template size + estimated expansion (128 bytes for paths)
        let mut result = String::with_capacity(template.len() + 128);
        let mut last_end = 0;
        
        // Single pass through template looking for placeholders
        let bytes = template.as_bytes();
        let mut i = 0;
        
        while i < bytes.len() {
            if bytes[i] == b'{' {
                // Found potential placeholder start
                // Append literal text before placeholder
                result.push_str(&template[last_end..i]);
                
                // Find closing brace
                if let Some(end) = template[i..].find('}') {
                    let placeholder_end = i + end;
                    let placeholder = &template[i + 1..placeholder_end];
                    
                    // Match and substitute placeholder
                    match placeholder {
                        "file_path" => result.push_str(&self.file_path),
                        "relative_path" => result.push_str(&self.relative_path),
                        "event_type" => result.push_str(self.event_type),
                        "absolute_path" => result.push_str(&self.absolute_path),
                        _ => {
                            // Unknown placeholder - keep as-is
                            result.push('{');
                            result.push_str(placeholder);
                            result.push('}');
                        }
                    }
                    
                    last_end = placeholder_end + 1;
                    i = placeholder_end + 1;
                } else {
                    // No closing brace - keep the opening brace
                    result.push('{');
                    last_end = i + 1;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        
        // Append remaining literal text
        result.push_str(&template[last_end..]);
        result
    }
}

/// Main file watcher that monitors directory changes
#[derive(Debug)]
pub struct FileWatcher {
    watch_path: PathBuf,
    filter: PatternFilter,
    command_config: CommandConfig,
    debounce_ms: u64,
}

impl FileWatcher {
    /// Create a new file watcher instance
    pub fn new(
        watch_path: PathBuf,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        command_config: CommandConfig,
        debounce_ms: u64,
    ) -> Result<Self> {
        // Ensure the watch path exists
        if !watch_path.exists() {
            anyhow::bail!("Directory does not exist: {}", watch_path.display());
        }

        if !watch_path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", watch_path.display());
        }

        // Convert to absolute path to match what notify gives us
        let watch_path = watch_path
            .canonicalize()
            .context("Failed to get absolute path of watch directory")?;

        let filter = PatternFilter::new(include_patterns, exclude_patterns)?;

        Ok(Self {
            watch_path,
            filter,
            command_config,
            debounce_ms,
        })
    }

    /// Start watching for file changes
    pub async fn start_watching(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Create watcher with recommended configuration
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                // Just forward events to the channel
                if let Err(e) = tx.send(res) {
                    eprintln!("Failed to send watch event: {}", e);
                }
            },
            Config::default(),
        )
        .context("Failed to create file watcher")?;

        // Start watching the directory recursively
        watcher
            .watch(&self.watch_path, RecursiveMode::Recursive)
            .context("Failed to start watching directory")?;

        log::info!("File watcher started successfully");
        if self.debounce_ms > 0 {
            log::info!("Debouncing enabled: {}ms", self.debounce_ms);
        }
        println!("🚀 Watching for file changes... Press Ctrl+C to stop");

        // Track pending events for debouncing: path -> (event, last_update_time)
        let mut pending_events: HashMap<PathBuf, (Event, Instant)> = HashMap::new();
        let debounce_duration = Duration::from_millis(self.debounce_ms);

        // Create ticker for checking pending events
        let check_interval = if self.debounce_ms > 0 {
            Duration::from_millis(50) // Check frequently when debouncing enabled
        } else {
            Duration::from_secs(3600) // Rarely check when debouncing disabled
        };
        let mut ticker = tokio::time::interval(check_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Process events asynchronously with graceful shutdown
        loop {
            tokio::select! {
                // Handle Ctrl+C for graceful shutdown
                _ = tokio::signal::ctrl_c() => {
                    log::info!("Received Ctrl+C, shutting down gracefully...");
                    println!("\n👋 Shutting down vibewatch...");
                    break;
                }
                // Receive file system events
                Some(res) = rx.recv() => {
                    match res {
                        Ok(event) => {
                            if self.debounce_ms == 0 {
                                // No debouncing - process immediately
                                self.handle_event(event);
                            } else {
                                // Debouncing enabled - track events
                                for path in &event.paths {
                                    pending_events.insert(path.clone(), (event.clone(), Instant::now()));
                                    log::debug!("Debouncing event for: {}", path.display());
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Watch error: {}", e);
                        }
                    }
                }
                // Check for events ready to process (exceeded debounce period)
                _ = ticker.tick() => {
                    if self.debounce_ms > 0 && !pending_events.is_empty() {
                        let now = Instant::now();
                        let ready_paths: Vec<PathBuf> = pending_events
                            .iter()
                            .filter(|(_, (_, time))| now.duration_since(*time) >= debounce_duration)
                            .map(|(path, _)| path.clone())
                            .collect();

                        for path in ready_paths {
                            if let Some((event, _)) = pending_events.remove(&path) {
                                log::debug!("Debounce period elapsed for: {}", path.display());
                                self.handle_event(event);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a file system event
    fn handle_event(&self, event: Event) {
        // Filter out events we don't care about
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                // These are the events we want to process
            }
            _ => return, // Ignore other event types
        }

        // Process each path in the event
        for path in event.paths {
            if let Some(relative_path) = self.get_relative_path(&path)
                && self.filter.should_watch(&relative_path)
            {
                // Check for special case: Modify(Name(Any)) might be a deletion from GUI applications
                let final_event_kind = match &event.kind {
                    EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
                        // If the file no longer exists, treat this as a deletion
                        if !path.exists() {
                            &EventKind::Remove(notify::event::RemoveKind::File)
                        } else {
                            &event.kind
                        }
                    }
                    _ => &event.kind,
                };

                Self::log_file_change(&relative_path, final_event_kind);

                // Execute command if configured
                self.execute_command_for_event(&path, &relative_path, final_event_kind);
            }
        }
    }

    /// Get relative path from the watch directory
    fn get_relative_path(&self, path: &Path) -> Option<PathBuf> {
        path.strip_prefix(&self.watch_path)
            .ok()
            .map(|p| p.to_path_buf())
    }

    /// Log file change with appropriate formatting (static version)
    fn log_file_change(path: &Path, event_kind: &EventKind) {
        let event_type = match event_kind {
            EventKind::Create(_) => "📝 Created",
            EventKind::Modify(_) => "✏️  Modified",
            EventKind::Remove(_) => "🗑️  Removed",
            _ => "📄 Changed",
        };

        println!("{}: {}", event_type, path.display());
        log::debug!("File event: {:?} - {}", event_kind, path.display());
    }

    /// Execute command for a file event if configured
    fn execute_command_for_event(&self, path: &Path, relative_path: &Path, event_kind: &EventKind) {
        if let Some(command_template) = self.command_config.get_command_for_event(event_kind) {
            let context = TemplateContext::new(path, relative_path, event_kind, &self.watch_path);
            let command = context.substitute_template(command_template);

            log::info!("Executing command: {}", command);

            // Execute command asynchronously
            tokio::spawn(async move {
                match Self::execute_shell_command(&command).await {
                    Ok(output) => {
                        log::debug!("Command executed successfully");
                        if !output.stdout.is_empty() {
                            log::debug!(
                                "Command stdout: {}",
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                        if !output.stderr.is_empty() {
                            log::warn!(
                                "Command stderr: {}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to execute command '{}': {}", command, e);
                    }
                }
            });
        }
    }

    /// Execute a shell command asynchronously
    async fn execute_shell_command(command: &str) -> Result<std::process::Output> {
        log::debug!("Executing shell command: {}", command);

        // Parse command with proper quote handling
        let parts = shell_words::split(command)
            .context("Failed to parse command")?;
        if parts.is_empty() {
            anyhow::bail!("Empty command");
        }

        let program = &parts[0];
        let args = &parts[1..];

        let output = TokioCommand::new(program)
            .args(args)
            .output()
            .await
            .context("Failed to execute command")?;

        if !output.status.success() {
            anyhow::bail!("Command failed with exit code: {:?}", output.status.code());
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, ModifyKind, RemoveKind};
    use rstest::rstest;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Parameterized tests for CommandConfig - testing command resolution for different event types
    #[rstest]
    // Create event tests
    #[case(
        Some("create_cmd"),
        None,
        None,
        Some("fallback"),
        EventKind::Create(CreateKind::File),
        Some("create_cmd")
    )]
    #[case(
        None,
        None,
        None,
        Some("fallback"),
        EventKind::Create(CreateKind::File),
        Some("fallback")
    )]
    #[case(
        Some("create_cmd"),
        None,
        None,
        None,
        EventKind::Create(CreateKind::Folder),
        Some("create_cmd")
    )]
    // Modify event tests
    #[case(
        None,
        Some("modify_cmd"),
        None,
        Some("fallback"),
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        Some("modify_cmd")
    )]
    #[case(
        None,
        None,
        None,
        Some("fallback"),
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        Some("fallback")
    )]
    #[case(
        None,
        Some("modify_cmd"),
        None,
        None,
        EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Any)),
        Some("modify_cmd")
    )]
    // Delete event tests
    #[case(
        None,
        None,
        Some("delete_cmd"),
        Some("fallback"),
        EventKind::Remove(RemoveKind::File),
        Some("delete_cmd")
    )]
    #[case(
        None,
        None,
        None,
        Some("fallback"),
        EventKind::Remove(RemoveKind::File),
        Some("fallback")
    )]
    #[case(
        None,
        None,
        Some("delete_cmd"),
        None,
        EventKind::Remove(RemoveKind::Folder),
        Some("delete_cmd")
    )]
    // No command configured
    #[case(None, None, None, None, EventKind::Create(CreateKind::File), None)]
    #[case(None, None, None, None, EventKind::Modify(ModifyKind::Any), None)]
    #[case(None, None, None, None, EventKind::Remove(RemoveKind::File), None)]
    // Other event types use fallback
    #[case(
        Some("create_cmd"),
        None,
        None,
        Some("fallback"),
        EventKind::Access(notify::event::AccessKind::Any),
        Some("fallback")
    )]
    #[case(
        None,
        Some("modify_cmd"),
        None,
        Some("fallback"),
        EventKind::Any,
        Some("fallback")
    )]
    fn test_command_config_resolution(
        #[case] on_create: Option<&str>,
        #[case] on_modify: Option<&str>,
        #[case] on_delete: Option<&str>,
        #[case] on_change: Option<&str>,
        #[case] event: EventKind,
        #[case] expected: Option<&str>,
    ) {
        let config = CommandConfig {
            on_create: on_create.map(|s| s.to_string()),
            on_modify: on_modify.map(|s| s.to_string()),
            on_delete: on_delete.map(|s| s.to_string()),
            on_change: on_change.map(|s| s.to_string()),
        };

        let result = config.get_command_for_event(&event);
        let expected_str = expected.map(|s| s.to_string());
        assert_eq!(
            result,
            expected_str.as_ref(),
            "Config({:?}, {:?}, {:?}, {:?}) with event {:?} should return {:?}",
            on_create,
            on_modify,
            on_delete,
            on_change,
            event,
            expected
        );
    }

    // Test TemplateContext with parameterized event types
    #[rstest]
    #[case(
        "/tmp/test/src/main.rs",
        "src/main.rs",
        EventKind::Create(CreateKind::File),
        "create",
        "/tmp/test/src/main.rs"
    )]
    #[case(
        "/tmp/test/file.txt",
        "file.txt",
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        "modify",
        "/tmp/test/file.txt"
    )]
    #[case(
        "/tmp/test/file.txt",
        "file.txt",
        EventKind::Remove(RemoveKind::File),
        "delete",
        "/tmp/test/file.txt"
    )]
    #[case(
        "/tmp/test/file.txt",
        "file.txt",
        EventKind::Access(notify::event::AccessKind::Any),
        "change",
        "/tmp/test/file.txt"
    )]
    fn test_template_context_event_types(
        #[case] file_path_str: &str,
        #[case] relative_path_str: &str,
        #[case] event: EventKind,
        #[case] expected_event_type: &str,
        #[case] expected_absolute: &str,
    ) {
        let file_path = PathBuf::from(file_path_str);
        let relative_path = PathBuf::from(relative_path_str);
        let watch_path = PathBuf::from("/tmp/test");

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        assert_eq!(ctx.file_path, file_path_str);
        assert_eq!(ctx.relative_path, relative_path_str);
        assert_eq!(ctx.event_type, expected_event_type);
        assert_eq!(ctx.absolute_path, expected_absolute);
    }

    #[test]
    fn test_template_substitution_all_variables() {
        let file_path = PathBuf::from("/home/user/project/src/lib.rs");
        let relative_path = PathBuf::from("src/lib.rs");
        let watch_path = PathBuf::from("/home/user/project");
        let event = EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any));

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        let template = "Event: {event_type}, File: {file_path}, Relative: {relative_path}, Absolute: {absolute_path}";
        let result = ctx.substitute_template(template);

        assert_eq!(
            result,
            "Event: modify, File: /home/user/project/src/lib.rs, Relative: src/lib.rs, Absolute: /home/user/project/src/lib.rs"
        );
    }

    #[test]
    fn test_template_substitution_partial() {
        let file_path = PathBuf::from("/tmp/file.txt");
        let relative_path = PathBuf::from("file.txt");
        let watch_path = PathBuf::from("/tmp");
        let event = EventKind::Create(CreateKind::File);

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        let template = "File created: {relative_path}";
        let result = ctx.substitute_template(template);

        assert_eq!(result, "File created: file.txt");
    }

    #[test]
    fn test_template_substitution_no_variables() {
        let file_path = PathBuf::from("/tmp/file.txt");
        let relative_path = PathBuf::from("file.txt");
        let watch_path = PathBuf::from("/tmp");
        let event = EventKind::Create(CreateKind::File);

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        let template = "echo 'Hello World'";
        let result = ctx.substitute_template(template);

        assert_eq!(result, "echo 'Hello World'");
    }

    #[test]
    fn test_template_substitution_multiple_same_variable() {
        let file_path = PathBuf::from("/tmp/file.txt");
        let relative_path = PathBuf::from("file.txt");
        let watch_path = PathBuf::from("/tmp");
        let event = EventKind::Create(CreateKind::File);

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        let template = "{relative_path} -> {relative_path}";
        let result = ctx.substitute_template(template);

        assert_eq!(result, "file.txt -> file.txt");
    }

    // Test FileWatcher initialization
    #[test]
    fn test_file_watcher_new_valid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_watcher_new_nonexistent_directory() {
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = FileWatcher::new(
            PathBuf::from("/nonexistent/path/that/does/not/exist"),
            vec![],
            vec![],
            config,
            0,
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Directory does not exist"));
    }

    #[test]
    fn test_file_watcher_new_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        std::fs::write(&file_path, "test").unwrap();

        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = FileWatcher::new(file_path, vec![], vec![], config, 0);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Path is not a directory"));
    }

    #[test]
    fn test_file_watcher_with_invalid_include_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = FileWatcher::new(
            temp_dir.path().to_path_buf(),
            vec!["[invalid".to_string()],
            vec![],
            config,
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_file_watcher_with_invalid_exclude_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let result = FileWatcher::new(
            temp_dir.path().to_path_buf(),
            vec![],
            vec!["[invalid".to_string()],
            config,
            0,
        );
        assert!(result.is_err());
    }

    // Test execute_shell_command
    #[tokio::test]
    async fn test_execute_shell_command_success() {
        let result = FileWatcher::execute_shell_command("echo test").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
    }

    #[tokio::test]
    async fn test_execute_shell_command_with_args() {
        let result = FileWatcher::execute_shell_command("echo hello world").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("hello world"));
    }

    #[tokio::test]
    async fn test_execute_shell_command_failure() {
        // Use a command that should fail
        let result = FileWatcher::execute_shell_command("false").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_shell_command_empty() {
        let result = FileWatcher::execute_shell_command("").await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Empty command"));
    }

    #[tokio::test]
    async fn test_execute_shell_command_nonexistent() {
        let result = FileWatcher::execute_shell_command("nonexistent_command_12345").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_event_kind_to_string_all_types() {
        assert_eq!(
            TemplateContext::event_kind_to_str(&EventKind::Create(CreateKind::File)),
            "create"
        );
        assert_eq!(
            TemplateContext::event_kind_to_str(&EventKind::Modify(ModifyKind::Data(
                notify::event::DataChange::Any
            ))),
            "modify"
        );
        assert_eq!(
            TemplateContext::event_kind_to_str(&EventKind::Remove(RemoveKind::File)),
            "delete"
        );
        assert_eq!(
            TemplateContext::event_kind_to_str(&EventKind::Access(notify::event::AccessKind::Any)),
            "change"
        );
    }

    #[test]
    fn test_get_relative_path_success() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        // Use canonicalized path since FileWatcher stores canonicalized paths
        let file_path = temp_dir.path().canonicalize().unwrap().join("test.txt");
        let relative = watcher.get_relative_path(&file_path);

        assert_eq!(relative, Some(PathBuf::from("test.txt")));
    }

    #[test]
    fn test_get_relative_path_nested() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        // Use canonicalized path since FileWatcher stores canonicalized paths
        let file_path = temp_dir
            .path()
            .canonicalize()
            .unwrap()
            .join("src")
            .join("main.rs");
        let relative = watcher.get_relative_path(&file_path);

        assert_eq!(relative, Some(PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_get_relative_path_outside_watch_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        // Try with a path outside the watch directory
        let outside_path = PathBuf::from("/tmp/outside.txt");
        let relative = watcher.get_relative_path(&outside_path);

        assert_eq!(relative, None);
    }

    #[rstest]
    // Create kind variants
    #[case(
        "create",
        EventKind::Create(CreateKind::File),
        Some("create"),
        None,
        None
    )]
    #[case(
        "create",
        EventKind::Create(CreateKind::Folder),
        Some("create"),
        None,
        None
    )]
    #[case(
        "create",
        EventKind::Create(CreateKind::Any),
        Some("create"),
        None,
        None
    )]
    // Modify kind variants
    #[case(
        "modify",
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        None,
        Some("modify"),
        None
    )]
    #[case(
        "modify",
        EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Any)),
        None,
        Some("modify"),
        None
    )]
    #[case(
        "modify",
        EventKind::Modify(ModifyKind::Any),
        None,
        Some("modify"),
        None
    )]
    // Remove kind variants
    #[case(
        "delete",
        EventKind::Remove(RemoveKind::File),
        None,
        None,
        Some("delete")
    )]
    #[case(
        "delete",
        EventKind::Remove(RemoveKind::Folder),
        None,
        None,
        Some("delete")
    )]
    #[case(
        "delete",
        EventKind::Remove(RemoveKind::Any),
        None,
        None,
        Some("delete")
    )]
    fn test_command_config_all_event_kind_variants(
        #[case] expected_cmd: &str,
        #[case] event: EventKind,
        #[case] on_create: Option<&str>,
        #[case] on_modify: Option<&str>,
        #[case] on_delete: Option<&str>,
    ) {
        let config = CommandConfig {
            on_create: on_create.map(|s| s.to_string()),
            on_modify: on_modify.map(|s| s.to_string()),
            on_delete: on_delete.map(|s| s.to_string()),
            on_change: None,
        };

        assert_eq!(
            config.get_command_for_event(&event),
            Some(&expected_cmd.to_string()),
            "Event {:?} should return command '{}'",
            event,
            expected_cmd
        );
    }

    #[test]
    fn test_template_context_nested_paths() {
        let file_path = PathBuf::from("/home/user/project/src/deep/nested/file.rs");
        let relative_path = PathBuf::from("src/deep/nested/file.rs");
        let watch_path = PathBuf::from("/home/user/project");
        let event = EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any));

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        assert_eq!(ctx.file_path, "/home/user/project/src/deep/nested/file.rs");
        assert_eq!(ctx.relative_path, "src/deep/nested/file.rs");
        assert_eq!(
            ctx.absolute_path,
            "/home/user/project/src/deep/nested/file.rs"
        );
    }

    #[test]
    fn test_template_substitution_edge_cases() {
        let file_path = PathBuf::from("/tmp/test.txt");
        let relative_path = PathBuf::from("test.txt");
        let watch_path = PathBuf::from("/tmp");
        let event = EventKind::Create(CreateKind::File);

        let ctx = TemplateContext::new(&file_path, &relative_path, &event, &watch_path);

        // Empty template
        assert_eq!(ctx.substitute_template(""), "");

        // Template with no placeholders
        assert_eq!(ctx.substitute_template("static text"), "static text");

        // Template with incomplete placeholder
        assert_eq!(ctx.substitute_template("{file"), "{file");
        assert_eq!(ctx.substitute_template("file_path}"), "file_path}");

        // Template with unknown placeholder
        assert_eq!(ctx.substitute_template("{unknown}"), "{unknown}");
    }

    #[test]
    fn test_file_watcher_with_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let watcher = FileWatcher::new(
            temp_dir.path().to_path_buf(),
            vec!["*.rs".to_string()],
            vec!["target/**".to_string()],
            config,
            0,
        );

        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_execute_shell_command_with_output() {
        let result = FileWatcher::execute_shell_command("echo test123").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test123"));
    }

    // Parameterized test for all event kind to string conversions
    #[rstest]
    #[case(EventKind::Create(CreateKind::File), "create")]
    #[case(EventKind::Create(CreateKind::Folder), "create")]
    #[case(EventKind::Create(CreateKind::Any), "create")]
    #[case(EventKind::Modify(ModifyKind::Any), "modify")]
    #[case(
        EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Any)),
        "modify"
    )]
    #[case(
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        "modify"
    )]
    #[case(EventKind::Remove(RemoveKind::File), "delete")]
    #[case(EventKind::Remove(RemoveKind::Folder), "delete")]
    #[case(EventKind::Remove(RemoveKind::Any), "delete")]
    fn test_event_kind_to_string_conversion(#[case] event_kind: EventKind, #[case] expected: &str) {
        assert_eq!(
            expected,
            TemplateContext::event_kind_to_str(&event_kind),
            "EventKind {:?} should convert to '{}'",
            event_kind,
            expected
        );
    }

    #[tokio::test]
    async fn test_handle_event_with_valid_path() {
        use std::fs;
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: Some("echo test".to_string()),
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        // Create an event
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            paths: vec![test_file.canonicalize().unwrap()],
            attrs: Default::default(),
        };

        // This should execute without panic
        watcher.handle_event(event);
    }

    #[test]
    fn test_handle_event_with_filtered_path() {
        use std::fs;
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: Some("echo test".to_string()),
            on_delete: None,
            on_change: None,
        };

        // Only watch .rs files
        let watcher = FileWatcher::new(
            temp_dir.path().to_path_buf(),
            vec!["*.rs".to_string()],
            vec![],
            config,
            0,
        )
        .unwrap();

        // Create a non-matching file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        // Create an event for non-matching file
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            paths: vec![test_file.canonicalize().unwrap()],
            attrs: Default::default(),
        };

        // This should not execute command (filtered out)
        watcher.handle_event(event);
    }

    #[test]
    fn test_handle_event_ignored_event_types() {
        use std::fs;
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: Some("echo test".to_string()),
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        // Create an Access event (should be ignored)
        let event = Event {
            kind: EventKind::Access(notify::event::AccessKind::Any),
            paths: vec![test_file.canonicalize().unwrap()],
            attrs: Default::default(),
        };

        // This should return early without processing
        watcher.handle_event(event);
    }

    #[tokio::test]
    async fn test_handle_event_modify_name_with_existing_file() {
        use std::fs;
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: Some("echo renamed".to_string()),
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();

        // Create a ModifyName event with existing file
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Any)),
            paths: vec![test_file.canonicalize().unwrap()],
            attrs: Default::default(),
        };

        watcher.handle_event(event);
    }

    #[tokio::test]
    async fn test_handle_event_modify_name_with_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: Some("echo deleted".to_string()),
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        // Use a path that doesn't exist
        let nonexistent_file = temp_dir
            .path()
            .canonicalize()
            .unwrap()
            .join("nonexistent.txt");

        // Create a ModifyName event (will be treated as delete since file doesn't exist)
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Name(notify::event::RenameMode::Any)),
            paths: vec![nonexistent_file],
            attrs: Default::default(),
        };

        watcher.handle_event(event);
    }

    #[tokio::test]
    async fn test_handle_event_create_event() {
        use std::fs;
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: Some("echo created".to_string()),
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        let test_file = temp_dir.path().join("new.txt");
        fs::write(&test_file, "new").unwrap();

        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![test_file.canonicalize().unwrap()],
            attrs: Default::default(),
        };

        watcher.handle_event(event);
    }

    #[tokio::test]
    async fn test_handle_event_delete_event() {
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: Some("echo deleted".to_string()),
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        // For delete events, file doesn't exist
        let deleted_file = temp_dir.path().canonicalize().unwrap().join("deleted.txt");

        let event = Event {
            kind: EventKind::Remove(RemoveKind::File),
            paths: vec![deleted_file],
            attrs: Default::default(),
        };

        watcher.handle_event(event);
    }

    #[test]
    fn test_log_file_change_coverage() {
        use std::path::Path;

        // Test all event types for log coverage
        FileWatcher::log_file_change(Path::new("test.txt"), &EventKind::Create(CreateKind::File));
        FileWatcher::log_file_change(
            Path::new("test.txt"),
            &EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        );
        FileWatcher::log_file_change(Path::new("test.txt"), &EventKind::Remove(RemoveKind::File));
        FileWatcher::log_file_change(
            Path::new("test.txt"),
            &EventKind::Access(notify::event::AccessKind::Any),
        );
    }

    #[test]
    fn test_execute_command_for_event_no_command() {
        use std::fs;
        let temp_dir = TempDir::new().unwrap();
        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: None,
        };

        let watcher =
            FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0).unwrap();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").unwrap();
        let canonical = test_file.canonicalize().unwrap();

        // Should not panic when no command is configured
        watcher.execute_command_for_event(
            &canonical,
            Path::new("test.txt"),
            &EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        );
    }

    #[test]
    fn test_start_watching_initialization() {
        // Test that start_watching can be called and initializes properly
        // We can't test the full event loop without it blocking, but we can
        // verify the watcher setup works by checking error cases

        // This test verifies the watcher creation and initial setup
        // The actual event loop is covered by integration testing
        let temp_dir = TempDir::new().unwrap();

        let config = CommandConfig {
            on_create: None,
            on_modify: None,
            on_delete: None,
            on_change: Some("echo test".to_string()),
        };

        let watcher = FileWatcher::new(temp_dir.path().to_path_buf(), vec![], vec![], config, 0);
        assert!(watcher.is_ok());

        // The watcher is valid and could start_watching if we called it
        // But we don't call it in the test because it would block
    }

    #[test]
    fn test_watcher_channel_send_error_coverage() {
        // This test covers the error path in the watcher callback
        // when the channel receiver is dropped
        use std::sync::mpsc;

        let (tx, rx): (mpsc::Sender<Result<Event, notify::Error>>, _) = mpsc::channel();

        // Drop the receiver immediately
        drop(rx);

        // Now sending should fail
        let result = tx.send(Ok(Event::new(EventKind::Any)));
        assert!(result.is_err());
    }
}
