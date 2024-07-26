use std::error::Error;

use crate::common::fs as cfs;
use crate::cpu::TimesStat;

const PROC_STAT: &str = "/proc/stat";
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
    let lines = cfs::read_lines_offset_n(PROC_STAT, 0, -1)?;

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

