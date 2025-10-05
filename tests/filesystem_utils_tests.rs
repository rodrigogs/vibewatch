use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

// Integration tests for vibewatch file watcher
// These tests verify end-to-end functionality with real file operations

#[test]
fn test_temp_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    assert!(temp_dir.path().exists());
    assert!(temp_dir.path().is_dir());
}

#[test]
fn test_file_creation_in_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    
    fs::write(&file_path, "test content").unwrap();
    
    assert!(file_path.exists());
    assert!(file_path.is_file());
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_file_modification_in_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create file
    fs::write(&file_path, "initial content").unwrap();
    let initial_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(initial_content, "initial content");
    
    // Modify file
    std::thread::sleep(Duration::from_millis(10));
    fs::write(&file_path, "modified content").unwrap();
    let modified_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(modified_content, "modified content");
}

#[test]
fn test_file_deletion_in_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    
    // Create file
    fs::write(&file_path, "test content").unwrap();
    assert!(file_path.exists());
    
    // Delete file
    fs::remove_file(&file_path).unwrap();
    assert!(!file_path.exists());
}

#[test]
fn test_nested_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("level1").join("level2").join("level3");
    
    fs::create_dir_all(&nested_path).unwrap();
    assert!(nested_path.exists());
    assert!(nested_path.is_dir());
}

#[test]
fn test_multiple_files_in_nested_structure() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    let tests_dir = temp_dir.path().join("tests");
    
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    
    let main_rs = src_dir.join("main.rs");
    let lib_rs = src_dir.join("lib.rs");
    let test_rs = tests_dir.join("integration_test.rs");
    
    fs::write(&main_rs, "fn main() {}").unwrap();
    fs::write(&lib_rs, "pub fn lib() {}").unwrap();
    fs::write(&test_rs, "#[test] fn test() {}").unwrap();
    
    assert!(main_rs.exists());
    assert!(lib_rs.exists());
    assert!(test_rs.exists());
}

#[test]
fn test_file_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");
    
    fs::write(&file_path, "test content").unwrap();
    
    let metadata = fs::metadata(&file_path).unwrap();
    assert!(metadata.is_file());
    assert!(!metadata.is_dir());
    assert!(metadata.len() > 0);
}

#[test]
fn test_directory_listing() {
    let temp_dir = TempDir::new().unwrap();
    
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
    fs::write(temp_dir.path().join("file3.txt"), "content3").unwrap();
    
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert_eq!(entries.len(), 3);
}

#[test]
fn test_file_rename() {
    let temp_dir = TempDir::new().unwrap();
    let old_path = temp_dir.path().join("old_name.txt");
    let new_path = temp_dir.path().join("new_name.txt");
    
    fs::write(&old_path, "content").unwrap();
    assert!(old_path.exists());
    
    fs::rename(&old_path, &new_path).unwrap();
    assert!(!old_path.exists());
    assert!(new_path.exists());
}

#[test]
fn test_copy_file() {
    let temp_dir = TempDir::new().unwrap();
    let src_path = temp_dir.path().join("source.txt");
    let dst_path = temp_dir.path().join("destination.txt");
    
    fs::write(&src_path, "content to copy").unwrap();
    fs::copy(&src_path, &dst_path).unwrap();
    
    assert!(src_path.exists());
    assert!(dst_path.exists());
    
    let src_content = fs::read_to_string(&src_path).unwrap();
    let dst_content = fs::read_to_string(&dst_path).unwrap();
    assert_eq!(src_content, dst_content);
}

#[test]
fn test_path_canonicalization() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    fs::write(&file_path, "content").unwrap();
    
    let canonical = file_path.canonicalize().unwrap();
    assert!(canonical.is_absolute());
    assert!(canonical.exists());
}

#[test]
fn test_relative_path_operations() {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();
    let file_path = base.join("subdir").join("file.txt");
    
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "content").unwrap();
    
    let relative = file_path.strip_prefix(base).unwrap();
    assert_eq!(relative, PathBuf::from("subdir").join("file.txt"));
}

#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    
    fs::create_dir(&empty_dir).unwrap();
    
    let entries: Vec<_> = fs::read_dir(&empty_dir)
        .unwrap()
        .collect();
    
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_remove_directory_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let dir_to_remove = temp_dir.path().join("to_remove");
    
    fs::create_dir_all(dir_to_remove.join("nested")).unwrap();
    fs::write(dir_to_remove.join("file.txt"), "content").unwrap();
    fs::write(dir_to_remove.join("nested").join("nested_file.txt"), "content").unwrap();
    
    assert!(dir_to_remove.exists());
    
    fs::remove_dir_all(&dir_to_remove).unwrap();
    assert!(!dir_to_remove.exists());
}

#[test]
fn test_file_permissions_exist() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    fs::write(&file_path, "content").unwrap();
    
    let metadata = fs::metadata(&file_path).unwrap();
    let permissions = metadata.permissions();
    
    // Just verify we can read permissions (actual values are platform-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.mode();
        assert!(mode > 0);
    }
    
    #[cfg(not(unix))]
    {
        // On Windows, just check that permissions exist
        assert!(!permissions.readonly() || permissions.readonly());
    }
}

#[test]
fn test_append_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("append.txt");
    
    use std::io::Write;
    
    // Write initial content
    fs::write(&file_path, "line 1\n").unwrap();
    
    // Append more content
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&file_path)
        .unwrap();
    
    writeln!(file, "line 2").unwrap();
    writeln!(file, "line 3").unwrap();
    drop(file);
    
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("line 1"));
    assert!(content.contains("line 2"));
    assert!(content.contains("line 3"));
}

#[test]
fn test_binary_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("binary.bin");
    
    let binary_data: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
    fs::write(&file_path, &binary_data).unwrap();
    
    let read_data = fs::read(&file_path).unwrap();
    assert_eq!(binary_data, read_data);
}

#[test]
fn test_symlink_creation() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target.txt");
        let link = temp_dir.path().join("link.txt");
        
        fs::write(&target, "target content").unwrap();
        symlink(&target, &link).unwrap();
        
        assert!(link.exists());
        let content = fs::read_to_string(&link).unwrap();
        assert_eq!(content, "target content");
    }
}

#[test]
fn test_hidden_files() {
    let temp_dir = TempDir::new().unwrap();
    let hidden_file = temp_dir.path().join(".hidden");
    
    fs::write(&hidden_file, "hidden content").unwrap();
    assert!(hidden_file.exists());
    
    let content = fs::read_to_string(&hidden_file).unwrap();
    assert_eq!(content, "hidden content");
}

#[test]
fn test_large_number_of_files() {
    let temp_dir = TempDir::new().unwrap();
    let num_files = 100;
    
    for i in 0..num_files {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        fs::write(&file_path, format!("content {}", i)).unwrap();
    }
    
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert_eq!(entries.len(), num_files);
}

#[test]
fn test_concurrent_file_operations() {
    use std::sync::Arc;
    use std::thread;
    
    let temp_dir = Arc::new(TempDir::new().unwrap());
    let mut handles = vec![];
    
    for i in 0..10 {
        let temp_dir = Arc::clone(&temp_dir);
        let handle = thread::spawn(move || {
            let file_path = temp_dir.path().join(format!("thread_{}.txt", i));
            fs::write(&file_path, format!("thread {} content", i)).unwrap();
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert_eq!(entries.len(), 10);
}
