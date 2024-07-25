use std::{io, mem};
use std::cmp::min;
use std::error::Error;

use windows::core::{BSTR, VARIANT, w};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, COINIT_MULTITHREADED, CoInitializeEx, CoInitializeSecurity, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE};
use windows::Win32::System::Wmi::{IWbemLocator, WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE, WbemLocator};
use windows_sys::Wdk::System::SystemInformation::{NtQuerySystemInformation, SystemProcessorPerformanceInformation};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows_sys::Win32::System::Threading::{ALL_PROCESSOR_GROUPS, GetActiveProcessorCount, GetSystemTimes};

use crate::common::binary::{little_endian_u32, little_endian_u64};
use crate::cpu::{InfoStat, TimesStat};

const DEFAULT_CPU_NUM: u32 = 1024;
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


pub fn total_cpu_times() -> Result<Vec<TimesStat>, Box<dyn Error>> {
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

    Ok(vec![TimesStat { cpu: "total".to_string(), user, system, idle, ..Default::default() }])
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

pub fn all_infos() -> Result<Vec<InfoStat>, Box<dyn Error>> {
    let mut ret = Vec::new();

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        )?;

        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;

        let server =
            locator.ConnectServer(&BSTR::from("root\\cimv2"), None, None, None, 0, None, None)?;

        let query = server.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from("select Family,Manufacturer,Name,NumberOfLogicalProcessors,NumberOfCores,ProcessorID,Stepping,MaxClockSpeed from Win32_Processor"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )?;


        let mut cpu = 0;
        loop {
            let mut row = vec![None];
            let mut returned = 0;
            query.Next(WBEM_INFINITE, &mut row, &mut returned).ok()?;

            if returned <= 0 {
                break;
            }

            if let Some(row) = &row[0] {
                let mut family = VARIANT::default();
                let mut manufacturer = VARIANT::default();
                let mut name = VARIANT::default();
                let mut number_of_logical_processors = VARIANT::default();
                let mut number_of_cores = VARIANT::default();
                let mut processor_id = VARIANT::default();
                let mut stepping = VARIANT::default();
                let mut max_clock_speed = VARIANT::default();

                row.Get(w!("Family"), 0, &mut family, None, None)?;
                row.Get(w!("Manufacturer"), 0, &mut manufacturer, None, None)?;
                row.Get(w!("Name"), 0, &mut name, None, None)?;
                row.Get(w!("NumberOfLogicalProcessors"), 0, &mut number_of_logical_processors, None, None)?;
                row.Get(w!("NumberOfCores"), 0, &mut number_of_cores, None, None)?;
                row.Get(w!("ProcessorID"), 0, &mut processor_id, None, None)?;
                row.Get(w!("Stepping"), 0, &mut stepping, None, None)?;
                row.Get(w!("MaxClockSpeed"), 0, &mut max_clock_speed, None, None)?;

                ret.push(InfoStat {
                    cpu,
                    vendor_id: manufacturer.to_string(),
                    family: family.to_string(),
                    physical_id: processor_id.to_string(),
                    cores: number_of_logical_processors.to_string().parse::<i32>()?,
                    model_name: name.to_string(),
                    mhz: max_clock_speed.to_string().parse::<f64>()?,
                    ..Default::default()
                });

                cpu += 1;
            } else {
                break;
            }
        }
    }

    Ok(ret)
}

pub fn logical_counts() -> Result<u32, Box<dyn Error>> {
    unsafe {
        let ret = GetActiveProcessorCount(ALL_PROCESSOR_GROUPS);
        if ret != 0 {
            return Ok(ret);
        }

        let mut si: SYSTEM_INFO = mem::zeroed();
        GetSystemInfo(&mut si);
        Ok(si.dwNumberOfProcessors)
    }
}

pub fn physical_counts() -> Result<u32, Box<dyn Error>> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        )?;

        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;

        let server =
            locator.ConnectServer(&BSTR::from("root\\cimv2"), None, None, None, 0, None, None)?;

        let query = server.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from("select NumberOfCores from Win32_Processor"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )?;


        let mut cpu = 0;
        loop {
            let mut row = vec![None];
            let mut returned = 0;
            query.Next(WBEM_INFINITE, &mut row, &mut returned).ok()?;

            if let Some(row) = &row[0] {
                let mut number_of_cores = VARIANT::default();

                row.Get(w!("NumberOfCores"), 0, &mut number_of_cores, None, None)?;

                cpu += number_of_cores.to_string().parse::<u32>().unwrap();
            } else {
                return Ok(cpu);
            }
        }
    }
}

fn performance_info() -> Result<Vec<Win32SystemProcessorPerformanceInformation>, Box<dyn Error>> {
    let cpu = logical_counts().unwrap_or(DEFAULT_CPU_NUM);
    let win32system_processor_performance_information_size = mem::size_of::<Win32SystemProcessorPerformanceInformation>();
    let length: u32 = win32system_processor_performance_information_size as u32 * cpu;
    let mut buffer: Vec<u8> = vec![0u8; length as usize];
    let mut ret = 0;
    unsafe {
        let status = NtQuerySystemInformation(SystemProcessorPerformanceInformation, buffer.as_mut_ptr() as _, length, &mut ret);
        if status != 0 {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("call to NtQuerySystemInformation returned {}.", status))));
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

