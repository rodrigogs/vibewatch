use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn template_substitution_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_substitution");

    // Simple template with one variable
    group.bench_function("single_variable", |b| {
        b.iter(|| {
            let template = black_box("echo {file_path}");
            let result = template.replace("{file_path}", "/home/user/project/src/main.rs");
            black_box(result);
        });
    });

    // Template with all variables
    group.bench_function("all_variables", |b| {
        b.iter(|| {
            let template = black_box("Event: {event_type}, File: {file_path}, Relative: {relative_path}, Absolute: {absolute_path}");
            let result = template
                .replace("{file_path}", "/home/user/project/src/main.rs")
                .replace("{relative_path}", "src/main.rs")
                .replace("{event_type}", "modify")
                .replace("{absolute_path}", "/home/user/project/src/main.rs");
            black_box(result);
        });
    });

    // Template with multiple occurrences
    group.bench_function("repeated_variable", |b| {
        b.iter(|| {
            let template = black_box("{file_path} -> {file_path}");
            let result = template.replace("{file_path}", "/home/user/project/src/main.rs");
            black_box(result);
        });
    });

    // Complex template (realistic command)
    group.bench_function("complex_command", |b| {
        b.iter(|| {
            let template = black_box("notify-send 'File Changed' 'Event: {event_type}\\nFile: {file_path}\\nPath: {relative_path}'");
            let result = template
                .replace("{file_path}", "/home/user/project/src/main.rs")
                .replace("{relative_path}", "src/main.rs")
                .replace("{event_type}", "modify")
                .replace("{absolute_path}", "/home/user/project/src/main.rs");
            black_box(result);
        });
    });

    // Benchmark with varying template lengths
    for size in [10, 50, 100, 200].iter() {
        let template = format!("echo {{file_path}} {}", "x".repeat(*size));
        group.bench_with_input(
            BenchmarkId::new("template_length", size),
            &template,
            |b, t| {
                b.iter(|| {
                    let result = t.replace("{file_path}", "/home/user/project/src/main.rs");
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn string_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // Compare old 4-pass vs conceptual single-pass
    let template = "Event: {event_type}, File: {file_path}, Relative: {relative_path}, Absolute: {absolute_path}";

    group.bench_function("four_pass_replace", |b| {
        b.iter(|| {
            let result = black_box(template)
                .replace("{file_path}", "/home/user/project/src/main.rs")
                .replace("{relative_path}", "src/main.rs")
                .replace("{event_type}", "modify")
                .replace("{absolute_path}", "/home/user/project/src/main.rs");
            black_box(result);
        });
    });

    // Simulate pre-allocation benefit
    group.bench_function("with_preallocation", |b| {
        b.iter(|| {
            let mut result = String::with_capacity(template.len() + 128);
            result.push_str(black_box(template));
            let result = result
                .replace("{file_path}", "/home/user/project/src/main.rs")
                .replace("{relative_path}", "src/main.rs")
                .replace("{event_type}", "modify")
                .replace("{absolute_path}", "/home/user/project/src/main.rs");
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    template_substitution_benchmark,
    string_operations_benchmark
);
criterion_main!(benches);
