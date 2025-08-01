// Performance and benchmark tests
use criterion::{criterion_group, criterion_main, Criterion};
use ferris_swarm_core::Chunk;

fn bench_chunk_creation(c: &mut Criterion) {
    // Create a temporary file for benchmarking
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path().to_path_buf();
    
    c.bench_function("chunk_creation", |b| {
        b.iter(|| {
            Chunk::new(
                temp_path.clone(),
                0,
                vec!["test_param".to_string()],
            )
        })
    });
}

fn bench_config_loading(c: &mut Criterion) {
    c.bench_function("config_default", |b| {
        b.iter(|| {
            ferris_swarm_config::Settings::default()
        })
    });
}

criterion_group!(benches, bench_chunk_creation, bench_config_loading);
criterion_main!(benches);