use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use lz4::Decoder;
use lz4::EncoderBuilder;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use rand::{thread_rng, Rng};
use std::io::{Read, Write};

const DATA_SIZES: [usize; 3] = [1024, 1024 * 1024, 1024 * 1024 * 10]; // 1KB, 1MB, 10MB

fn generate_binary_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    thread_rng().fill(&mut data[..]);
    data
}

// Gzip Compression & Decompression
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

// LZ4-Flex Compression & Decompression
fn lz4_flex_compression(data: &[u8]) -> Vec<u8> {
    compress_prepend_size(data)
}

fn lz4_flex_decompression(data: &[u8]) -> Vec<u8> {
    decompress_size_prepended(data).unwrap()
}

// LZ4 Compression & Decompression with Different Levels
fn lz4_compression(data: &[u8], level: u32) -> Vec<u8> {
    let mut encoder = EncoderBuilder::new()
        .level(level) // Set LZ4 compression level (0-16)
        .build(Vec::new())
        .unwrap();
    encoder.write_all(data).unwrap();
    let (compressed, result) = encoder.finish();
    result.unwrap();
    compressed
}

fn lz4_decompression(data: &[u8]) -> Vec<u8> {
    let mut decoder = Decoder::new(data).unwrap();
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

        // LZ4-Flex Compression Benchmark
        group.bench_function("lz4_flex", |b| {
            b.iter(|| {
                let compressed = lz4_flex_compression(black_box(&data));
                black_box(compressed);
            })
        });

        // LZ4 Compression Benchmarks at Different Levels
        group.bench_function("lz4_fast", |b| {
            b.iter(|| {
                let compressed = lz4_compression(black_box(&data), 0); // Fastest mode
                black_box(compressed);
            })
        });

        group.bench_function("lz4_default", |b| {
            b.iter(|| {
                let compressed = lz4_compression(black_box(&data), 4); // Default mode
                black_box(compressed);
            })
        });

        group.bench_function("lz4_best", |b| {
            b.iter(|| {
                let compressed = lz4_compression(black_box(&data), 16); // Best compression
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

        // LZ4-Flex Decompression Benchmark
        let lz4_flex_compressed = lz4_flex_compression(&data);
        group.bench_function("lz4_flex_decompress", |b| {
            b.iter(|| {
                let decompressed = lz4_flex_decompression(black_box(&lz4_flex_compressed));
                black_box(decompressed);
            })
        });

        // LZ4 Decompression Benchmark at Different Levels
        let lz4_compressed_fast = lz4_compression(&data, 0);
        group.bench_function("lz4_decompress_fast", |b| {
            b.iter(|| {
                let decompressed = lz4_decompression(black_box(&lz4_compressed_fast));
                black_box(decompressed);
            })
        });

        let lz4_compressed_default = lz4_compression(&data, 4);
        group.bench_function("lz4_decompress_default", |b| {
            b.iter(|| {
                let decompressed = lz4_decompression(black_box(&lz4_compressed_default));
                black_box(decompressed);
            })
        });

        let lz4_compressed_best = lz4_compression(&data, 16);
        group.bench_function("lz4_decompress_best", |b| {
            b.iter(|| {
                let decompressed = lz4_decompression(black_box(&lz4_compressed_best));
                black_box(decompressed);
            })
        });

        group.finish();
    }
}

criterion_group!(benches, benchmark_compression_speed);
criterion_main!(benches);
