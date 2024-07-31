use std::cmp::min;
use std::error::Error;

use crate::common::fs as cfs;
use crate::mem::VirtualMemoryStat;

const PROC_MEMINFO: &str = "/proc/meminfo";
const PROC_ZONEINFO: &str = "/proc/zoneinfo";


#[derive(Default, Debug)]
struct ExVirtualMemory {
    active_file: u64,
    inactive_file: u64,
    active_anon: u64,
    inactive_anon: u64,
    unevictable: u64,
}

pub fn get_virtual_memory() -> Result<VirtualMemoryStat, Box<dyn Error>> {
    let lines = cfs::read_lines(PROC_MEMINFO)?;
    let mut ret = VirtualMemoryStat { ..Default::default() };
    let mut ret_ex = ExVirtualMemory { ..Default::default() };

    for line in lines {
        let fields = line.split(":").map(String::from).collect::<Vec<String>>();
        if fields.len() < 2 {
            continue;
        }

        let key = fields[0].trim();
        let value = fields[1].trim().replace(" kB", "");

        if let Ok(mut v) = value.parse::<u64>() {
            // v = v * 1024
            v = v << 10;

            match key {
                "MemTotal" => ret.total = v,
                "MemFree" => ret.free = v,
                "MemAvailable" => ret.available = v,
                "Buffers" => ret.buffers = v,
                "Cached" => ret.cached = v,
                "Active" => ret.active = v,
                "Inactive" => ret.inactive = v,
                "Active(anon)" => ret_ex.active_anon = v,
                "Inactive(anon)" => ret_ex.inactive_anon = v,
                "Active(file)" => ret_ex.active_file = v,
                "Inactive(file)" => ret_ex.inactive_file = v,
                "Unevictable" => ret_ex.unevictable = v,
                "Writeback" => ret.write_back = v,
                "WritebackTmp" => ret.write_back_tmp = v,
                "Dirty" => ret.dirty = v,
                "Shmem" => ret.shared = v,
                "Slab" => ret.slab = v,
                "SReclaimable" => ret.sreclaimable = v,
                "SUnreclaim" => ret.sunreclaim = v,
                "PageTables" => ret.page_tables = v,
                "SwapCached" => ret.swap_cached = v,
                "CommitLimit" => ret.commit_limit = v,
                "Committed_AS" => ret.committed_as = v,
                "HighTotal" => ret.high_total = v,
                "HighFree" => ret.high_free = v,
                "LowTotal" => ret.low_total = v,
                "LowFree" => ret.low_free = v,
                "SwapTotal" => ret.swap_total = v,
                "SwapFree" => ret.swap_free = v,
                "Mapped" => ret.mapped = v,
                "VmallocTotal" => ret.vmalloc_total = v,
                "VmallocUsed" => ret.vmalloc_used = v,
                "VmallocChunk" => ret.vmalloc_chunk = v,
                "HugePages_Total" => ret.huge_pages_total = v,
                "HugePages_Free" => ret.huge_pages_free = v,
                "HugePages_Rsvd" => ret.huge_pages_rsvd = v,
                "HugePages_Surp" => ret.huge_pages_surp = v,
                "Hugepagesize" => ret.huge_page_size = v,
                "AnonHugePages" => ret.anon_huge_pages = v,
                _ => {}
            }
        }
    }

    ret.cached += ret.sreclaimable;
    if ret.available > 0 {
        if ret_ex.active_file > 0 && ret_ex.inactive_file > 0 && ret.sreclaimable > 0 {
            ret.available = calculate_avail_vmem(&ret, &ret_ex);
        } else {
            ret.available = ret.cached + ret.free;
        }
    }

    ret.used = ret.total - ret.free - ret.buffers - ret.cached;
    ret.used_percent = ret.used as f64 / ret.total as f64 * 100.0;

    Ok(ret)
}

fn calculate_avail_vmem(ret: &VirtualMemoryStat, ret_ex: &ExVirtualMemory) -> u64 {
    if let Ok(lines) = cfs::read_lines(PROC_ZONEINFO) {
        let mut watermark_low = 0;

        for line in lines {
            let mut fields = line.split_ascii_whitespace();
            if fields.next().unwrap_or("").starts_with("low") {
                watermark_low += fields.next().unwrap_or("").parse::<u64>().unwrap_or(0);
            }
        }

        watermark_low += page_size() as u64;
        let mut avali_memory = ret.free - watermark_low;
        let mut page_cache = ret_ex.active_file + ret_ex.inactive_file;
        page_cache -= min(page_cache / 2, watermark_low);
        avali_memory += page_cache;
        avali_memory += ret.sreclaimable - min(ret.sreclaimable / 2, watermark_low);

        return if avali_memory < 0 { 0 } else { avali_memory };
    }

    ret.free + ret.cached
}

pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}
