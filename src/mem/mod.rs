#[derive(Default, Debug)]
pub struct VirtualMemoryStat {
    // Total amount of RAM on this system
    pub total: u64,

    // RAM available for programs to allocate
    //
    // This value is computed from the kernel specific values.
    pub available: u64,

    // RAM used by programs
    //
    // This value is computed from the kernel specific values.
    pub used: u64,

    // Percentage of RAM used by programs
    //
    // This value is computed from the kernel specific values.
    pub used_percent: f64,

    // This is the kernel's notion of free memory; RAM chips whose bits nobody
    // cares about the value of right now. For a human consumable number,
    // Available is what you really want.
    pub free: u64,

    // OS X / BSD specific numbers:
    // http://www.macyourself.com/2010/02/17/what-is-free-wired-active-and-inactive-system-memory-ram/
    pub active: u64,
    pub inactive: u64,
    pub wired: u64,

    // FreeBSD specific numbers:
    // https://reviews.freebsd.org/D8467
    pub laundry: u64,

    // Linux specific numbers
    // https://www.centos.org/docs/5/html/5.1/Deployment_Guide/s2-proc-meminfo.html
    // https://www.kernel.org/doc/Documentation/filesystems/proc.txt
    // https://www.kernel.org/doc/Documentation/vm/overcommit-accounting
    // https://www.kernel.org/doc/Documentation/vm/transhuge.txt
    pub buffers: u64,
    pub cached: u64,
    pub write_back: u64,
    pub dirty: u64,
    pub write_back_tmp: u64,
    pub shared: u64,
    pub slab: u64,
    pub sreclaimable: u64,
    pub sunreclaim: u64,
    pub page_tables: u64,
    pub swap_cached: u64,
    pub commit_limit: u64,
    pub committed_as: u64,
    pub high_total: u64,
    pub high_free: u64,
    pub low_total: u64,
    pub low_free: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub mapped: u64,
    pub vmalloc_total: u64,
    pub vmalloc_used: u64,
    pub vmalloc_chunk: u64,
    pub huge_pages_total: u64,
    pub huge_pages_free: u64,
    pub huge_pages_rsvd: u64,
    pub huge_pages_surp: u64,
    pub huge_page_size: u64,
    pub anon_huge_pages: u64,
}

#[derive(Default, Debug)]
pub struct SwapMemoryStat {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub used_percent: f64,
    pub sin: u64,
    pub sout: u64,
    pub pg_in: u64,
    pub pg_out: u64,
    pub pg_fault: u64,
    // Linux specific numbers
    // https://www.kernel.org/doc/Documentation/cgroup-v2.txt
    pub pg_maj_fault: u64,
}

#[derive(Default, Debug)]
pub struct SwapDevice {
    name: String,
    used_bytes: u64,
    free_bytes: u64,
}