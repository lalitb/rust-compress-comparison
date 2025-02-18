use flate2::write::GzEncoder;
use flate2::Compression;
use lz4::EncoderBuilder;
use lz4_flex::compress_prepend_size;
use rand::{distributions::Alphanumeric, Rng, thread_rng};
use std::io::Write;
use std::time::Instant;

const DATA_SIZE: usize = 1024 * 1024 * 10; // 10MB
const NUM_TRIALS: usize = 1;

// Different types of test data
enum TestData {
    Random,
    Repeating,
    Mixed,
}

fn generate_test_data(data_type: &TestData) -> Vec<u8> {
    match data_type {
        TestData::Random => thread_rng()
            .sample_iter(&Alphanumeric)
            .take(DATA_SIZE)
            .map(|c| c as u8)
            .collect(),

        TestData::Repeating => {
            let pattern = b"HelloWorld";
            let mut data = Vec::with_capacity(DATA_SIZE);
            while data.len() < DATA_SIZE {
                data.extend_from_slice(pattern);
            }
            data.truncate(DATA_SIZE);
            data
        },

        TestData::Mixed => {
            let mut data = Vec::with_capacity(DATA_SIZE);
            let mut rng = thread_rng();

            while data.len() < DATA_SIZE {
                if rng.gen_bool(0.3) {
                    data.extend_from_slice(b"HelloWorld");
                } else {
                    data.push(rng.sample(Alphanumeric) as u8);
                }
            }
            data.truncate(DATA_SIZE);
            data
        },
    }
}

// Gzip Compression
fn gzip_compression(data: &[u8], level: Compression) -> (Vec<u8>, f64) {
    let start = Instant::now();
    let mut encoder = GzEncoder::new(Vec::new(), level);
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    let duration = start.elapsed().as_secs_f64();
    (compressed, duration)
}

// LZ4-Flex Compression
fn lz4_flex_compression(data: &[u8]) -> (Vec<u8>, f64) {
    let start = Instant::now();
    let compressed = compress_prepend_size(data);
    let duration = start.elapsed().as_secs_f64();
    (compressed, duration)
}

// LZ4-RS Compression (lz4 crate) with Different Levels
fn lz4_rs_compression(data: &[u8], level: u32) -> (Vec<u8>, f64) {
    let start = Instant::now();
    let mut encoder = EncoderBuilder::new()
        .level(level) // LZ4 compression level (0-16)
        .build(Vec::new())
        .unwrap();
    encoder.write_all(data).unwrap();
    let (compressed, result) = encoder.finish();
    result.unwrap();
    let duration = start.elapsed().as_secs_f64();
    (compressed, duration)
}

// Struct to Store Benchmark Results
#[derive(Default)]
struct CompressionStats {
    factor_sum: f64,
    time_sum: f64,
    size_sum: usize,
}

fn main() {
    let compression_levels = [
        ("Fast", Compression::fast()),
        ("Default", Compression::default()),
        ("Best", Compression::best()),
    ];

    let lz4_rs_levels = [
        ("Fast", 0),
        ("Default", 4),
        ("Best", 16),
    ];

    let test_cases = [
        ("Random", TestData::Random),
        ("Repeating", TestData::Repeating),
        ("Mixed", TestData::Mixed),
    ];

    println!("\nRunning compression benchmarks ({} trials of {}MB data)...\n",
             NUM_TRIALS, DATA_SIZE / 1024 / 1024);

    for (data_name, data_type) in &test_cases {
        println!("=== {} Data ===", data_name);

        // GZip Benchmarks
        for (level_name, level) in &compression_levels {
            let mut stats = CompressionStats::default();

            for _ in 0..NUM_TRIALS {
                let data = generate_test_data(data_type);
                let original_size = data.len();
                let (compressed, duration) = gzip_compression(&data, *level);
                let compressed_size = compressed.len();

                let factor = original_size as f64 / compressed_size as f64;
                stats.factor_sum += factor;
                stats.time_sum += duration;
                stats.size_sum += compressed_size;
            }

            println!("\n--- Gzip {} ---\nCompression Factor: {:.2}x | Time: {:.3}s | Avg Size: {:.2}MB",
                level_name,
                stats.factor_sum / NUM_TRIALS as f64,
                stats.time_sum / NUM_TRIALS as f64,
                (stats.size_sum / NUM_TRIALS) as f64 / (1024.0 * 1024.0)
            );
        }

        // LZ4-Flex Benchmarks
        let mut stats = CompressionStats::default();
        for _ in 0..NUM_TRIALS {
            let data = generate_test_data(data_type);
            let original_size = data.len();
            let (compressed, duration) = lz4_flex_compression(&data);
            let compressed_size = compressed.len();

            let factor = original_size as f64 / compressed_size as f64;
            stats.factor_sum += factor;
            stats.time_sum += duration;
            stats.size_sum += compressed_size;
        }

        println!(
            "\n--- LZ4-Flex Compression ({} data) ---\nCompression Factor: {:.2}x | Time: {:.3}s | Avg Size: {:.2}MB",
            data_name,
            stats.factor_sum / NUM_TRIALS as f64,
            stats.time_sum / NUM_TRIALS as f64,
            (stats.size_sum / NUM_TRIALS) as f64 / (1024.0 * 1024.0)
        );

        // LZ4-RS Benchmarks at Multiple Levels
        for (level_name, level) in &lz4_rs_levels {
            let mut stats = CompressionStats::default();
            for _ in 0..NUM_TRIALS {
                let data = generate_test_data(data_type);
                let original_size = data.len();
                let (compressed, duration) = lz4_rs_compression(&data, *level);
                let compressed_size = compressed.len();

                let factor = original_size as f64 / compressed_size as f64;
                stats.factor_sum += factor;
                stats.time_sum += duration;
                stats.size_sum += compressed_size;
            }

            println!(
                "\n--- LZ4-RS {} Compression ({} data) ---\nCompression Factor: {:.2}x | Time: {:.3}s | Avg Size: {:.2}MB",
                level_name,
                data_name,
                stats.factor_sum / NUM_TRIALS as f64,
                stats.time_sum / NUM_TRIALS as f64,
                (stats.size_sum / NUM_TRIALS) as f64 / (1024.0 * 1024.0)
            );
        }
    }
}
