use std::{io, mem};
use std::cmp::min;
use std::error::Error;

use windows_sys::Wdk::System::SystemInformation::{NtQuerySystemInformation, SystemProcessorPerformanceInformation};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows_sys::Win32::System::Threading::GetSystemTimes;

use crate::common::binary::{little_endian_u32, little_endian_u64};
use crate::cpu::TimesStat;

const DEFAULT_CPU_NUM: usize = 1024;
const CLOCKS_PER_SEC: f64 = 10000000.0;

#[derive(Debug)]
struct Win32SystemProcessorPerformanceInformation {
    idle_time: i64,
    kernel_time: i64,
    user_time: i64,
    dpc_time: i64,
    interrupt_time: i64,
    interrupt_count: u32,
}


pub fn total_times() -> Result<TimesStat, Box<dyn Error>> {
    let mut lpidletime = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut lpkerneltime = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let mut lpusertime = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

    let ret = unsafe {
        GetSystemTimes(&mut lpidletime, &mut lpkerneltime, &mut lpusertime)
    };

    if ret == 0 {
        return Err(io::Error::last_os_error().into());
    }

    let lot = 0.0000001;
    let hit = 429.4967296;
    let idle = (hit * lpidletime.dwHighDateTime as f64) + (lot * lpidletime.dwLowDateTime as f64);
    let user = (hit * lpusertime.dwHighDateTime as f64) + (lot * lpusertime.dwLowDateTime as f64);
    let kernel = (hit * lpkerneltime.dwHighDateTime as f64) + (lot * lpkerneltime.dwLowDateTime as f64);
    let system = kernel - idle;

    Ok(TimesStat { cpu: "total".to_string(), user, system, idle, ..Default::default() })
}

pub fn cpu_num() -> Option<usize> {
    unsafe {
        let mut si: SYSTEM_INFO = mem::zeroed();
        GetSystemInfo(&mut si);
        Some(si.dwNumberOfProcessors as usize)
    }
}

pub fn per_cpu_times() -> Result<Vec<TimesStat>, Box<dyn Error>> {
    let performances = performance_info()?;
    let mut result = Vec::with_capacity(performances.len());

    for (i, p) in performances.iter().enumerate() {
        result.push(TimesStat {
            cpu: format!("cpu{}", i),
            user: p.user_time as f64 / CLOCKS_PER_SEC,
            system: (p.kernel_time - p.idle_time) as f64 / CLOCKS_PER_SEC,
            idle: p.idle_time as f64 / CLOCKS_PER_SEC,
            irq: p.interrupt_time as f64 / CLOCKS_PER_SEC,
            ..Default::default()
        })
    }

    Ok(result)
}

fn performance_info() -> Result<Vec<Win32SystemProcessorPerformanceInformation>, Box<dyn Error>> {
    let cpu = cpu_num().unwrap_or(DEFAULT_CPU_NUM);
    let win32system_processor_performance_information_size = mem::size_of::<Win32SystemProcessorPerformanceInformation>();
    let length: u32 = (win32system_processor_performance_information_size * cpu) as u32;
    let mut buffer: Vec<u8> = vec![0u8; length as usize];
    let mut ret = 0;
    unsafe {
        let status = NtQuerySystemInformation(SystemProcessorPerformanceInformation, buffer.as_mut_ptr() as _, length, &mut ret);
        if status != 0 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("call to NtQuerySystemInformation returned {}.", status))));
        }
    }

    // calculate the number of returned elements based on the returned size
    let nums = ret / win32system_processor_performance_information_size as u32;

    let mut result = Vec::with_capacity(nums as usize);
    let end = min(nums * win32system_processor_performance_information_size as u32, buffer.len() as u32);
    for i in (0..end).step_by(win32system_processor_performance_information_size) {
        let i = i as usize;
        let info = Win32SystemProcessorPerformanceInformation {
            idle_time: little_endian_u64(&buffer[i..i + 8]) as i64,
            kernel_time: little_endian_u64(&buffer[i + 8..i + 16]) as i64,
            user_time: little_endian_u64(&buffer[i + 16..i + 24]) as i64,
            dpc_time: little_endian_u64(&buffer[i + 24..i + 32]) as i64,
            interrupt_time: little_endian_u64(&buffer[i + 32..i + 40]) as i64,
            interrupt_count: little_endian_u32(&buffer[i + 40..i + 44]),
        };
        result.push(info)
    }


    Ok(result)
}

