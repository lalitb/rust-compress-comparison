use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use rand::{Rng, thread_rng};
use std::io::{Write, Read};

const DATA_SIZES: [usize; 3] = [1024, 1024 * 1024, 1024 * 1024 * 10]; // 1KB, 1MB, 10MB

fn generate_binary_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    thread_rng().fill(&mut data[..]);
    data
}

fn gzip_compression(data: &[u8], level: Compression) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), level);
    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

fn gzip_decompression(data: &[u8]) -> Vec<u8> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

fn lz4_flex_compression(data: &[u8]) -> Vec<u8> {
    compress_prepend_size(data)
}

fn lz4_flex_decompression(data: &[u8]) -> Vec<u8> {
    decompress_size_prepended(data).unwrap()
}

// lz4 Compression & Decompression
fn lz4_compression(data: &[u8]) -> Vec<u8> {
    let mut encoder = lz4::EncoderBuilder::new()
        .level(4) // Compression level (0-16)
        .build(Vec::new())
        .unwrap();
    encoder.write_all(data).unwrap();
    let (compressed, result) = encoder.finish();
    result.unwrap();
    compressed
}

fn lz4_decompression(data: &[u8]) -> Vec<u8> {
    let mut decoder = lz4::Decoder::new(data).unwrap();
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

fn benchmark_compression_speed(c: &mut Criterion) {
    for &size in &DATA_SIZES {
        let data = generate_binary_data(size); // Generate once per size
        let mut group = c.benchmark_group(format!("Compression_{}B", size));

        // Gzip Compression Benchmarks
        group.bench_function("gzip_fast", |b| {
            b.iter(|| {
                let compressed = gzip_compression(black_box(&data), Compression::fast());
                black_box(compressed);
            })
        });

        group.bench_function("gzip_default", |b| {
            b.iter(|| {
                let compressed = gzip_compression(black_box(&data), Compression::default());
                black_box(compressed);
            })
        });

        group.bench_function("gzip_best", |b| {
            b.iter(|| {
                let compressed = gzip_compression(black_box(&data), Compression::best());
                black_box(compressed);
            })
        });

        // LZ4-flex Compression Benchmark
        group.bench_function("lz4-flex", |b| {
            b.iter(|| {
                let compressed = lz4_flex_compression(black_box(&data));
                black_box(compressed);
            })
        });

        // lz4 Compression Benchmark
        group.bench_function("lz4", |b| {
            b.iter(|| {
                let compressed = lz4_compression(black_box(&data));
                black_box(compressed);
            })
        });

        // Gzip Decompression Benchmark
        let gzip_compressed = gzip_compression(&data, Compression::default());
        group.bench_function("gzip_decompress", |b| {
            b.iter(|| {
                let decompressed = gzip_decompression(black_box(&gzip_compressed));
                black_box(decompressed);
            })
        });

        // LZ4-flex ecompression Benchmark
        let lz4_compressed = lz4_flex_compression(&data);
        group.bench_function("lz4_decompress", |b| {
            b.iter(|| {
                let decompressed = lz4_flex_decompression(black_box(&lz4_compressed));
                black_box(decompressed);
            })
        });

        // lz4 Decompression Benchmark
        let lz4_compressed = lz4_compression(&data);
        group.bench_function("lz4_decompress", |b| {
            b.iter(|| {
                let decompressed = lz4_decompression(black_box(&lz4_compressed));
                black_box(decompressed);
            })
        });
    


        group.finish();
    }
}

criterion_group!(benches, benchmark_compression_speed);
criterion_main!(benches);
