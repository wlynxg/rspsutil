use std::cmp::min;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::mem::MaybeUninit;

use crate::common::fs as cfs;
use crate::mem::{SwapDevice, SwapMemoryStat, VirtualMemoryStat};

const PROC_MEMINFO: &str = "/proc/meminfo";
const PROC_ZONEINFO: &str = "/proc/zoneinfo";
const PROC_VMSTAT: &str = "/proc/vmstat";
const PROC_SWAPS: &str = "/proc/swaps";


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

pub fn get_swap_memory() -> Result<SwapMemoryStat, Box<dyn Error>> {
    let info = sys_info()?;
    let mut ret = SwapMemoryStat {
        total: info.totalswap * info.mem_unit as u64,
        free: info.freeswap * info.mem_unit as u64,
        ..Default::default()
    };
    ret.used = ret.total - ret.free;

    // check Infinity
    if ret.total != 0 {
        ret.used_percent = ret.used as f64 / ret.total as f64 * 100.0;
    } else {
        ret.used_percent = 0.0;
    }


    let lines = cfs::read_lines(PROC_VMSTAT)?;
    let kv = lines.iter().filter_map(|line| {
        let mut parts = line.split_whitespace();
        let key = parts.next()?;
        let value = parts.next()?.parse::<u64>().ok()?;
        Some((key, value))
    }).collect::<HashMap<_, _>>();
    let get_value = |key: &str| kv.get(key).copied().unwrap_or(0) << 12;

    ret.sin = get_value("pswpin");
    ret.sout = get_value("pswpout");
    ret.pg_in = get_value("pgpgin");
    ret.pg_out = get_value("pgpgout");
    ret.pg_fault = get_value("pgpgfault");
    ret.pg_maj_fault = get_value("pgmajfault");
    Ok(ret)
}

pub fn get_swap_devices() -> Result<Vec<SwapDevice>, Box<dyn Error>> {
    let lines = cfs::read_lines(PROC_SWAPS)?;

    if lines.len() < 2 {
        return Ok(vec![]);
    }

    let headers = lines[0].split_ascii_whitespace().
        into_iter().map(String::from).collect::<Vec<String>>();

    if headers.len() < 3 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("couldn't parse {PROC_SWAPS}: too few fields in header"))));
    }

    if headers[0] != "Filename" || headers[2] != "Size" || headers[3] != "Used" {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("couldn't parse {PROC_SWAPS}: headers unexpected fields"))));
    }

    let mut ret = vec![];
    for line in lines[1..].iter() {
        let fields = line.split_ascii_whitespace().
            into_iter().map(String::from).collect::<Vec<String>>();
        if fields.len() < 3 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("couldn't parse {PROC_SWAPS}: too few fields in header"))));
        }


        let total_kb = fields[2].parse::<u64>()?;
        let used_kb = fields[3].parse::<u64>()?;

        ret.push(SwapDevice {
            name: fields[0].clone(),
            used_bytes: used_kb << 10,
            free_bytes: (total_kb - used_kb) << 10,
        })
    }

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

fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

fn sys_info() -> Result<libc::sysinfo, Box<dyn Error>> {
    let mut info = MaybeUninit::uninit();
    let ret = unsafe { libc::sysinfo(info.as_mut_ptr()) };

    let info = unsafe { info.assume_init() };
    if ret == 0 {
        Ok(info)
    } else {
        Err(io::Error::last_os_error().into())
    }
}