use std::error::Error;

use crate::common::fs as cfs;
use crate::cpu::{InfoStat, TimesStat};

const PROC_STAT: &str = "/proc/stat";
const PROC_CPUINFO: &str = "/proc/cpuinfo";
const SYS_CPU: &str = "/sys/devices/system/cpu";
const CLOCKS_PER_SEC: f64 = 100.0;

pub fn total_cpu_times() -> Result<Vec<TimesStat>, Box<dyn Error>> {
    let lines = cfs::read_lines_offset_n(PROC_STAT, 0, 1)?;

    let mut ret = Vec::with_capacity(lines.len());
    for line in lines {
        let state = parse_stat_line(&line)?;
        ret.push(state)
    }

    Ok(ret)
}

pub fn per_cpu_times() -> Result<Vec<TimesStat>, Box<dyn Error>> {
    let lines = cfs::read_lines(PROC_STAT)?;

    let mut ret = Vec::new();
    if lines.len() < 2 {
        return Ok(ret);
    }

    for line in lines[1..].iter() {
        if !line.starts_with("cpu") {
            break;
        }

        let stat = parse_stat_line(line)?;
        ret.push(stat)
    }

    Ok(ret)
}

pub fn all_infos() -> Result<Vec<InfoStat>, Box<dyn Error>> {
    let lines = cfs::read_lines(PROC_CPUINFO)?;

    let mut stat = InfoStat { cpu: -1, cores: 1, ..Default::default() };
    let mut process_name = String::new();
    let mut ret: Vec<InfoStat> = Vec::new();

    for i in lines {
        let fields: Vec<String> = i.split(":").map(String::from).collect();
        if fields.len() < 2 {
            continue;
        }


        let key = fields[0].trim();
        let value = fields[1].trim().to_string();

        match key {
            "Processor" => process_name = value,
            "processor" | "cpu number" => {
                if stat.cpu >= 0 {
                    finish_cpu_info(&mut stat);
                    ret.push(stat);
                }

                stat = InfoStat { cores: 1, model_name: process_name.clone(), ..Default::default() };
                let t = value.parse::<i32>()?;
                stat.cpu = t;
            }
            "vendorId" | "vendor_id" => {
                if value.contains("S390") {
                    process_name = "S390".to_string();
                }
                stat.vendor_id = value
            }
            "CPU implementer" => {
                if let Ok(v) = usize::from_str_radix(value.as_str(), 8) {
                    stat.vendor_id = String::from(match v {
                        0x41 => "ARM",
                        0x42 => "Broadcom",
                        0x43 => "Cavium",
                        0x44 => "DEC",
                        0x46 => "Fujitsu",
                        0x48 => "HiSilicon",
                        0x49 => "Infineon",
                        0x4d => "Motorola/Freescale",
                        0x4e => "NVIDIA",
                        0x50 => "APM",
                        0x51 => "Qualcomm",
                        0x56 => "Marvell",
                        0x61 => "Apple",
                        0x69 => "Intel",
                        0xc0 => "Ampere",
                        _ => ""
                    });
                }
            }
            "cpu family" => {
                stat.family = value
            }
            "model" | "CPU part" => {
                stat.model = value;
                // if CPU is arm based, model name is found via model number. refer to: arch/arm64/kernel/cpuinfo.c
                if stat.vendor_id == "ARM" {
                    if let Ok(v) = usize::from_str_radix(&stat.model, 0) {
                        stat.model_name = String::from(match v {
                            0x810 => "ARM810",
                            0x920 => "ARM920",
                            0x922 => "ARM922",
                            0x926 => "ARM926",
                            0x940 => "ARM940",
                            0x946 => "ARM946",
                            0x966 => "ARM966",
                            0xa20 => "ARM1020",
                            0xa22 => "ARM1022",
                            0xa26 => "ARM1026",
                            0xb02 => "ARM11 MPCore",
                            0xb36 => "ARM1136",
                            0xb56 => "ARM1156",
                            0xb76 => "ARM1176",
                            0xc05 => "Cortex-A5",
                            0xc07 => "Cortex-A7",
                            0xc08 => "Cortex-A8",
                            0xc09 => "Cortex-A9",
                            0xc0d => "Cortex-A17",
                            0xc0f => "Cortex-A15",
                            0xc0e => "Cortex-A17",
                            0xc14 => "Cortex-R4",
                            0xc15 => "Cortex-R5",
                            0xc17 => "Cortex-R7",
                            0xc18 => "Cortex-R8",
                            0xc20 => "Cortex-M0",
                            0xc21 => "Cortex-M1",
                            0xc23 => "Cortex-M3",
                            0xc24 => "Cortex-M4",
                            0xc27 => "Cortex-M7",
                            0xc60 => "Cortex-M0+",
                            0xd01 => "Cortex-A32",
                            0xd02 => "Cortex-A34",
                            0xd03 => "Cortex-A53",
                            0xd04 => "Cortex-A35",
                            0xd05 => "Cortex-A55",
                            0xd06 => "Cortex-A65",
                            0xd07 => "Cortex-A57",
                            0xd08 => "Cortex-A72",
                            0xd09 => "Cortex-A73",
                            0xd0a => "Cortex-A75",
                            0xd0b => "Cortex-A76",
                            0xd0c => "Neoverse-N1",
                            0xd0d => "Cortex-A77",
                            0xd0e => "Cortex-A76AE",
                            0xd13 => "Cortex-R52",
                            0xd20 => "Cortex-M23",
                            0xd21 => "Cortex-M33",
                            0xd40 => "Neoverse-V1",
                            0xd41 => "Cortex-A78",
                            0xd42 => "Cortex-A78AE",
                            0xd43 => "Cortex-A65AE",
                            0xd44 => "Cortex-X1",
                            0xd46 => "Cortex-A510",
                            0xd47 => "Cortex-A710",
                            0xd48 => "Cortex-X2",
                            0xd49 => "Neoverse-N2",
                            0xd4a => "Neoverse-E1",
                            0xd4b => "Cortex-A78C",
                            0xd4c => "Cortex-X1C",
                            0xd4d => "Cortex-A715",
                            0xd4e => "Cortex-X3",
                            _ => "Undefined"
                        })
                    }
                }
            }
            "Model Name" | "model name" | "cpu" => {
                if value.contains("POWER") {
                    stat.model = String::from(value.split_whitespace().next().unwrap_or(""));
                    stat.family = String::from("POWER");
                    stat.vendor_id = String::from("IBM");
                }
                stat.model_name = value;
            }
            "stepping" | "revision" | "CPU revision" => {
                let val: String = if key == "revision" { value.split(".").next().unwrap_or("").to_string() } else { value };
                stat.stepping = val.parse::<i32>()?;
            }
            "cpu MHz" | "clock" | "cpu MHz dynamic" => {
                // treat this as the fallback value, thus we ignore error
                if let Ok(v) = value.replace("MHz", "").parse::<_>() {
                    stat.mhz = v;
                }
            }
            "cache size" => {
                if let Ok(v) = value.replace(" KB", "").parse::<_>() {
                    stat.cache_size = v;
                }
            }
            "physical id" => {
                stat.physical_id = value;
            }
            "core id" => {
                stat.core_id = value;
            }
            "flags" | "Features" => {
                stat.flags = value.split(|x| x == ',' || x == ' ').map(String::from).collect();
            }
            "microcode" => {
                stat.microcode = value;
            }
            &_ => {}
        }
    }

    if stat.cpu >= 0 {
        finish_cpu_info(&mut stat);
        ret.push(stat);
    }

    Ok(ret)
}

fn parse_stat_line(line: &str) -> Result<TimesStat, Box<dyn Error>> {
    let fields: Vec<String> = line.split_whitespace().map(String::from).collect();
    if fields.len() < 8 {
        return Err("stat does not contain cpu info".into());
    }

    if !fields[0].starts_with("cpu") {
        return Err("not contain cpu".into());
    }

    let cpu = if fields[0].eq("cpu") { String::from("cpu-total") } else { fields[0].to_string() };
    let user: f64 = fields[1].parse::<f64>()? / CLOCKS_PER_SEC;
    let nice: f64 = fields[2].parse::<f64>()? / CLOCKS_PER_SEC;
    let system: f64 = fields[3].parse::<f64>()? / CLOCKS_PER_SEC;
    let idle: f64 = fields[4].parse::<f64>()? / CLOCKS_PER_SEC;
    let io_wait: f64 = fields[5].parse::<f64>()? / CLOCKS_PER_SEC;
    let irq: f64 = fields[6].parse::<f64>()? / CLOCKS_PER_SEC;
    let soft_irq: f64 = fields[7].parse::<f64>()? / CLOCKS_PER_SEC;

    // Linux >= 2.6.11
    let steal = if fields.len() > 8 { fields[8].parse::<f64>()? as f64 / CLOCKS_PER_SEC } else { 0.0 };

    // Linux >= 2.6.24
    let guest = if fields.len() > 9 { fields[9].parse::<f64>()? as f64 / CLOCKS_PER_SEC } else { 0.0 };

    // Linux >= 3.2.0
    let guest_nice = if fields.len() > 10 { fields[10].parse::<f64>()? as f64 / CLOCKS_PER_SEC } else { 0.0 };


    Ok(TimesStat {
        cpu,
        user,
        nice,
        system,
        idle,
        io_wait,
        irq,
        soft_irq,
        steal,
        guest,
        guest_nice,
    })
}

fn finish_cpu_info(stat: &mut InfoStat) {
    if stat.core_id.len() == 0 {
        if let Ok(v) = cfs::read_lines(&format!("{}/cpu{}/topology/core_id", SYS_CPU, stat.cpu)) {
            stat.core_id = v[0].clone();
        }
    }

    // override the value of c.Mhz with cpufreq/cpuinfo_max_freq regardless
    // of the value from /proc/cpuinfo because we want to report the maximum
    // clock-speed of the CPU for c.Mhz, matching the behaviour of Windows
    if let Ok(lines) = cfs::read_lines(&format!("{}/cpu{}/cpufreq/cpuinfo_max_freq", SYS_CPU, stat.cpu)) {
        // if we encounter errors below such as there are no cpuinfo_max_freq file,
        // we just ignore. so let Mhz is 0.
        if lines.len() == 0 {
            return;
        }

        if let Ok(v) = lines[0].parse::<f64>() {
            if stat.mhz > 9999_000.0 {
                stat.mhz = v / 1000_000.0;
            } else {
                stat.mhz = v / 1000.0;
            }
        }
    }
}

