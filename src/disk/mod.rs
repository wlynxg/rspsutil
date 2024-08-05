struct UsageStat {
    path: String,
    fs_type: String,
    total: u64,
    free: u64,
    used: u64,
    used_percent: f64,
    inodes_total: u64,
    inodes_used: u64,
    inodes_free: u64,
    inodes_used_percent: f64,
}

struct PartitionStat {
    device: String,
    mountpoint: String,
    fstype: String,
    opts: Vec<String>,
}

struct IOCountersStat {
    read_count: u64,
    merged_read_count: u64,
    write_count: u64,
    merged_write_count: u64,
    read_bytes: u64,
    write_bytes: u64,
    read_time: u64,
    write_time: u64,
    iops_in_progress: u64,
    io_time: u64,
    weighted_io: u64,
    name: String,
    serial_number: String,
    label: String,
}
