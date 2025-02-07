use super::rocksdb::{ColumnFamilyOptions, DBOptions, WriteOptions, DB};
use std::time::Instant;

/// 运行 WAL 测试，每次写入 1024 字节的 value，循环指定次数
fn run_bench_wal(name: &str, mut opts: DBOptions, wopts: WriteOptions, iterations: usize) {
    // 创建临时目录
    let path = tempfile::Builder::new()
        .prefix(name)
        .tempdir()
        .expect("创建临时目录失败");
    let path_str = path.path().to_str().unwrap();

    opts.create_if_missing(true);
    opts.set_max_background_jobs(6);
    opts.set_max_subcompactions(2);

    // 设置 ColumnFamilyOptions
    let mut cf_opts = ColumnFamilyOptions::new();
    cf_opts.set_write_buffer_size(16 * 1024);
    cf_opts.set_max_write_buffer_number(10);

    // 打开数据库，使用 "default" 列族
    let db = DB::open_cf(opts, path_str, vec![("default", cf_opts)])
        .expect("打开数据库失败");

    let value = vec![1; 1024];
    let start = Instant::now();

    // 循环写入数据
    for i in 0..iterations {
        let key = format!("key_{}", i);
        db.put_opt(key.as_bytes(), &value, &wopts)
            .expect("写入数据失败");
    }
    let duration = start.elapsed();
    println!(
        "Benchmark '{}' ran {} iterations in {:?}",
        name, iterations, duration
    );
    // 关闭数据库
    drop(db);
}

/// 根据是否启用日志回收设置选项，并运行测试
fn run_bench_wal_recycle_log(name: &str, recycled: bool, iterations: usize) {
    let mut opts = DBOptions::new();
    if recycled {
        opts.set_recycle_log_file_num(10);
    }

    let mut wopts = WriteOptions::new();
    wopts.set_sync(true);

    run_bench_wal(name, opts, wopts, iterations);
}

#[test]
fn test_wal_with_recycle_log() {
    // 测试启用日志回收的情况
    run_bench_wal_recycle_log("_rust_rocksdb_wal_with_recycle_log", true, 1000);
}

#[test]
fn test_wal_without_recycle_log() {
    // 测试不启用日志回收的情况
    run_bench_wal_recycle_log("_rust_rocksdb_wal_without_recycle_log", false, 1000);
}

#[test]
fn test_wal_no_sync() {
    // 测试写入时不同步刷盘
    let opts = DBOptions::new();
    let mut wopts = WriteOptions::new();
    wopts.set_sync(false);

    run_bench_wal("_rust_rocksdb_wal_no_sync", opts, wopts, 1000);
}

#[test]
fn test_wal_disable_wal() {
    // 测试禁用 WAL 的情况
    let opts = DBOptions::new();
    let mut wopts = WriteOptions::new();
    wopts.disable_wal(true);

    run_bench_wal("_rust_rocksdb_wal_disable_wal", opts, wopts, 1000);
}
