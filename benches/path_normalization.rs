use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::path::PathBuf;

fn path_normalization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_normalization");

    // Unix-style paths (no backslashes)
    let unix_paths = vec![
        "/home/user/project/src/main.rs",
        "/usr/local/bin/app",
        "/var/log/system.log",
        "/home/user/documents/file.txt",
        "/tmp/test/deep/nested/path/file.rs",
    ];

    // Windows-style paths (with backslashes)
    let windows_paths = vec![
        "C:\\Users\\user\\project\\src\\main.rs",
        "C:\\Program Files\\app\\bin\\app.exe",
        "D:\\Documents\\file.txt",
        "C:\\Windows\\System32\\config.sys",
        "E:\\Projects\\deep\\nested\\path\\file.rs",
    ];

    // Benchmark Unix paths with always-replace strategy
    group.bench_function("unix_always_replace", |b| {
        b.iter(|| {
            for path in &unix_paths {
                let result = black_box(path).replace('\\', "/");
                black_box(result);
            }
        });
    });

    // Benchmark Unix paths with conditional replace (optimized)
    group.bench_function("unix_conditional_replace", |b| {
        b.iter(|| {
            for path in &unix_paths {
                let path_str = black_box(*path);
                let result = if path_str.contains('\\') {
                    path_str.replace('\\', "/")
                } else {
                    path_str.to_string()
                };
                black_box(result);
            }
        });
    });

    // Benchmark Windows paths with always-replace strategy
    group.bench_function("windows_always_replace", |b| {
        b.iter(|| {
            for path in &windows_paths {
                let result = black_box(path).replace('\\', "/");
                black_box(result);
            }
        });
    });

    // Benchmark Windows paths with conditional replace
    group.bench_function("windows_conditional_replace", |b| {
        b.iter(|| {
            for path in &windows_paths {
                let path_str = black_box(*path);
                let result = if path_str.contains('\\') {
                    path_str.replace('\\', "/")
                } else {
                    path_str.to_string()
                };
                black_box(result);
            }
        });
    });

    // Benchmark with varying path lengths
    for length in [20, 50, 100, 200].iter() {
        let unix_path = format!("/home/user/{}", "a/".repeat(*length / 2));
        let windows_path = format!("C:\\\\Users\\\\{}", "a\\\\".repeat(*length / 2));

        group.bench_with_input(
            BenchmarkId::new("unix_path_length", length),
            &unix_path,
            |b, path| {
                b.iter(|| {
                    let result = if path.contains('\\') {
                        path.replace('\\', "/")
                    } else {
                        path.to_string()
                    };
                    black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("windows_path_length", length),
            &windows_path,
            |b, path| {
                b.iter(|| {
                    let result = if path.contains('\\') {
                        path.replace('\\', "/")
                    } else {
                        path.to_string()
                    };
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn pathbuf_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathbuf_operations");

    let paths: Vec<PathBuf> = vec![
        PathBuf::from("/home/user/project/src/main.rs"),
        PathBuf::from("/usr/local/bin/app"),
        PathBuf::from("/var/log/system.log"),
    ];

    group.bench_function("display_to_string", |b| {
        b.iter(|| {
            for path in &paths {
                let result = black_box(path).display().to_string();
                black_box(result);
            }
        });
    });

    group.bench_function("display_to_string_and_normalize", |b| {
        b.iter(|| {
            for path in &paths {
                let path_str = black_box(path).display().to_string();
                let result = if path_str.contains('\\') {
                    path_str.replace('\\', "/")
                } else {
                    path_str
                };
                black_box(result);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    path_normalization_benchmark,
    pathbuf_operations_benchmark
);
criterion_main!(benches);
