#[cfg(windows)]
pub use windows::*;

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