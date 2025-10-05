use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use glob::Pattern;
use std::path::Path;

fn pattern_matching_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");
    
    // Common patterns
    let patterns = vec![
        "*.rs",
        "**/*.rs",
        "src/**/*.rs",
        "**/*.{rs,toml}",
        "**/target/**",
    ];
    
    // Test paths
    let paths = vec![
        "src/main.rs",
        "src/lib.rs",
        "tests/integration_test.rs",
        "target/debug/build/lib.rs",
        "Cargo.toml",
        "README.md",
        "src/utils/helper.rs",
        "benches/benchmark.rs",
    ];
    
    // Benchmark pattern compilation
    group.bench_function("pattern_compilation", |b| {
        b.iter(|| {
            for pattern_str in &patterns {
                let pattern = Pattern::new(black_box(pattern_str)).unwrap();
                black_box(pattern);
            }
        });
    });
    
    // Benchmark pattern matching
    for pattern_str in &patterns {
        let pattern = Pattern::new(pattern_str).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("match", pattern_str),
            &pattern,
            |b, pat| {
                b.iter(|| {
                    for path in &paths {
                        let result = pat.matches(black_box(path));
                        black_box(result);
                    }
                });
            },
        );
    }
    
    // Benchmark with compiled patterns (cached)
    let compiled_patterns: Vec<Pattern> = patterns
        .iter()
        .map(|p| Pattern::new(p).unwrap())
        .collect();
    
    group.bench_function("multiple_patterns_compiled", |b| {
        b.iter(|| {
            for path in &paths {
                for pattern in &compiled_patterns {
                    let result = pattern.matches(black_box(path));
                    black_box(result);
                }
            }
        });
    });
    
    // Benchmark pattern matching with Path types
    let path_objects: Vec<&Path> = paths.iter().map(|p| Path::new(p)).collect();
    
    group.bench_function("match_with_path_objects", |b| {
        b.iter(|| {
            for path in &path_objects {
                for pattern in &compiled_patterns {
                    let path_str = path.to_str().unwrap();
                    let result = pattern.matches(black_box(path_str));
                    black_box(result);
                }
            }
        });
    });
    
    group.finish();
}

fn exclude_pattern_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("exclude_patterns");
    
    // Common exclude patterns
    let exclude_patterns = vec![
        "**/target/**",
        "**/.git/**",
        "**/node_modules/**",
        "**/*.tmp",
        "**/.*",
    ];
    
    let compiled_excludes: Vec<Pattern> = exclude_patterns
        .iter()
        .map(|p| Pattern::new(p).unwrap())
        .collect();
    
    // Test paths (some should be excluded)
    let test_paths = vec![
        "src/main.rs",              // Should not be excluded
        "target/debug/lib.rs",      // Should be excluded
        ".git/config",              // Should be excluded
        "node_modules/package.json",// Should be excluded
        "temp.tmp",                 // Should be excluded
        ".hidden",                  // Should be excluded
        "tests/test.rs",            // Should not be excluded
    ];
    
    // Benchmark exclude logic
    group.bench_function("check_excludes", |b| {
        b.iter(|| {
            for path in &test_paths {
                let is_excluded = compiled_excludes
                    .iter()
                    .any(|pattern| pattern.matches(black_box(path)));
                black_box(is_excluded);
            }
        });
    });
    
    // Benchmark include + exclude logic (realistic scenario)
    let include_pattern = Pattern::new("**/*.rs").unwrap();
    
    group.bench_function("include_and_exclude", |b| {
        b.iter(|| {
            for path in &test_paths {
                let matches_include = include_pattern.matches(black_box(path));
                let is_excluded = compiled_excludes
                    .iter()
                    .any(|pattern| pattern.matches(black_box(path)));
                let should_watch = matches_include && !is_excluded;
                black_box(should_watch);
            }
        });
    });
    
    group.finish();
}

fn glob_alternatives_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("glob_alternatives");
    
    let pattern_str = "**/*.rs";
    let pattern = Pattern::new(pattern_str).unwrap();
    let path = "src/utils/helper.rs";
    
    // Simple string comparison (baseline)
    group.bench_function("string_ends_with", |b| {
        b.iter(|| {
            let result = black_box(path).ends_with(".rs");
            black_box(result);
        });
    });
    
    // Glob pattern matching
    group.bench_function("glob_pattern_match", |b| {
        b.iter(|| {
            let result = pattern.matches(black_box(path));
            black_box(result);
        });
    });
    
    // Complex pattern
    let complex_pattern = Pattern::new("src/**/*.{rs,toml}").unwrap();
    
    group.bench_function("complex_glob_match", |b| {
        b.iter(|| {
            let result = complex_pattern.matches(black_box(path));
            black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    pattern_matching_benchmark,
    exclude_pattern_benchmark,
    glob_alternatives_benchmark
);
criterion_main!(benches);
