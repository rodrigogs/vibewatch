# Performance Benchmarks

This directory contains Criterion-based benchmarks for vibewatch's performance-critical operations.

## Running Benchmarks

Run all benchmarks:
```bash
cargo bench
```

Run specific benchmark:
```bash
cargo bench --bench template_substitution
cargo bench --bench path_normalization
cargo bench --bench pattern_matching
```

Run with filter:
```bash
cargo bench -- template_length
```

## Benchmark Suites

### 1. Template Substitution (`template_substitution.rs`)

Benchmarks the template variable substitution system that replaces placeholders like `{file_path}` in commands.

**Key Metrics:**
- Single variable substitution performance
- All variables substitution (4 replacements)
- Repeated variable handling
- Complex command templates
- Performance vs template length

**Optimizations Measured:**
- 4-pass vs single-pass algorithm
- Pre-allocation benefits
- String operation overhead

### 2. Path Normalization (`path_normalization.rs`)

Benchmarks path normalization (backslash to forward slash conversion) with platform-specific optimizations.

**Key Metrics:**
- Unix paths (no backslashes) - optimized path
- Windows paths (with backslashes) - full replacement
- Conditional vs always-replace strategies
- Performance vs path length
- PathBuf conversion overhead

**Optimizations Measured:**
- Conditional `.contains()` check vs always calling `.replace()`
- Platform-specific behavior
- String allocation patterns

### 3. Pattern Matching (`pattern_matching.rs`)

Benchmarks glob pattern matching for include/exclude filtering.

**Key Metrics:**
- Pattern compilation cost
- Pattern matching performance
- Multiple pattern evaluation
- Include + exclude logic
- Complex glob patterns vs simple string operations

**Scenarios:**
- Common patterns (`*.rs`, `**/*.rs`, `**/target/**`)
- Exclude pattern evaluation
- Real-world include/exclude combinations

## Interpreting Results

Criterion generates detailed reports in `target/criterion/`:
- HTML reports with graphs
- Statistical analysis (mean, median, std dev)
- Performance change detection
- Regression warnings

Example output:
```
template_substitution/single_variable
                        time:   [45.123 ns 45.456 ns 45.789 ns]
                        change: [-5.2% -3.8% -2.1%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## Performance Targets

Based on optimization work (Steps 1-6):

| Component | Target | Achieved |
|-----------|--------|----------|
| Template substitution | ~50% faster | ✓ Single-pass algorithm |
| Path normalization | ~20-30% faster (Unix) | ✓ Conditional replace |
| Event processing | 10-20% lower latency | ✓ Async channels |
| Memory allocations | 40-60% fewer | ✓ Multiple optimizations |

## Continuous Monitoring

Run benchmarks regularly to:
- Detect performance regressions
- Validate optimization impact
- Compare implementation strategies
- Track performance over time

Use baseline comparisons:
```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Compare against baseline later
cargo bench -- --baseline main
```

## Adding New Benchmarks

1. Create new file in `benches/`
2. Add `[[bench]]` entry to `Cargo.toml`:
   ```toml
   [[bench]]
   name = "my_benchmark"
   harness = false
   ```
3. Use Criterion structure:
   ```rust
   use criterion::{criterion_group, criterion_main, Criterion};
   
   fn my_benchmark(c: &mut Criterion) {
       c.bench_function("my_test", |b| {
           b.iter(|| {
               // Code to benchmark
           });
       });
   }
   
   criterion_group!(benches, my_benchmark);
   criterion_main!(benches);
   ```

## Best Practices

1. **Use `black_box()`** to prevent compiler optimizations from eliminating code
2. **Warm up** operations before benchmarking (Criterion does this automatically)
3. **Minimize noise** by closing other applications
4. **Run multiple times** for statistical significance
5. **Compare against baselines** to track changes
6. **Document expectations** in commit messages

## Related Documentation

- `docs/PERFORMANCE_OPTIMIZATION.md` - Optimization strategies and analysis
- `docs/TESTING.md` - Testing approaches and patterns
- Criterion documentation: https://bheisler.github.io/criterion.rs/book/
