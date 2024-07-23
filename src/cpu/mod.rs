use std::error::Error;

use crate::cpu::windows::{all_infos, logical_counts, per_cpu_times, physical_counts, total_times};

#[cfg(windows)]
mod windows;


#[derive(Default, Debug)]
pub struct TimesStat {
    pub cpu: String,
    pub user: f64,
    pub system: f64,
    pub idle: f64,
    pub nice: f64,
    pub io_wait: f64,
    pub irq: f64,
    pub soft_irq: f64,
    pub steal: f64,
    pub guest: f64,
    pub guest_nice: f64,
}

#[derive(Default, Debug)]
pub struct InfoStat {
    pub cpu: i32,
    pub vendor_id: String,
    pub family: String,
    pub model: String,
    pub stepping: i32,
    pub physical_id: String,
    pub core_id: String,
    pub cores: i32,
    pub model_name: String,
    pub mhz: f64,
    pub cache_size: i32,
    pub flags: Vec<String>,
    pub microcode: String,
}


pub fn times(percpu: bool) -> Result<Vec<TimesStat>, Box<dyn Error>> {
    return if percpu {
        per_cpu_times()
    } else {
        total_times()
    };
}

pub fn infos() -> Result<Vec<InfoStat>, Box<dyn Error>> {
    all_infos()
}

pub fn counts(logical: bool) -> Result<u32, Box<dyn Error>> {
    return if logical {
        logical_counts()
    } else {
        physical_counts()
    };
}
