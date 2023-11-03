use serde::{Deserialize, Serialize};
use std::fs;
use systemstat::{ByteSize, Platform};

/// Struct representing network usage information for a specific interface.
#[derive(Deserialize, Serialize)]
pub struct NetworkUsage {
    interface: String,
    tx: String,
    rx: String,
    total: String,
}

impl NetworkUsage {
    /// Retrieves a list of available network interfaces.
    pub fn get_net_if() -> Vec<String> {
        let paths = fs::read_dir("/sys/class/net").unwrap();

        let mut interfaces: Vec<String> = Vec::new();

        for path in paths {
            interfaces.push(
                path.unwrap()
                    .path()
                    .display()
                    .to_string()
                    .replace("/sys/class/net/", ""),
            );
        }

        interfaces
    }

    /// Retrieves network usage details for all available interfaces.
    pub fn get_usage() -> Vec<NetworkUsage> {
        let mut network_usage_list: Vec<NetworkUsage> = Vec::new();

        for interface in NetworkUsage::get_net_if() {
            let interface_rx: u64 =
                fs::read_to_string(format!("/sys/class/net/{interface}/statistics/rx_bytes"))
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap(); /* It can be unwrapped, we won't get errors here */

            let interface_tx: u64 =
                fs::read_to_string(format!("/sys/class/net/{interface}/statistics/tx_bytes"))
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();

            network_usage_list.push(NetworkUsage::new(interface, interface_tx, interface_rx));
        }

        network_usage_list
    }

    /// Creates a new NetworkUsage instance with calculated total data.
    pub fn new(interface: String, tx: u64, rx: u64) -> NetworkUsage {
        let total: u64 = tx + rx;

        NetworkUsage {
            interface,
            tx: ByteSize(tx).to_string(),
            rx: ByteSize(rx).to_string(),
            total: ByteSize(total).to_string(),
        }
    }
}

/// Represents information about system uptime.
#[derive(Deserialize, Serialize)]
pub struct UptimeInfo {
    seconds: u64,
    pretty: String,
}

impl UptimeInfo {
    /// Creates a new `UptimeInfo` instance based on the given number of seconds.
    pub fn new(seconds: u64) -> UptimeInfo {
        UptimeInfo {
            seconds,
            pretty: UptimeInfo::seconds_to_pretty(seconds),
        }
    }

    /// Converts a duration in seconds to a human-readable string format.
    fn seconds_to_pretty(seconds: u64) -> String {
        let mut time_string = String::new();
        let mut _remaining = seconds;

        //up 4 days, 15 hours, 22 minutes
        let months: u64 = _remaining / 2592000;
        _remaining -= months * 2592000;

        let weeks: u64 = _remaining / 604800;
        _remaining -= weeks / 604800;

        let days: u64 = _remaining / 86400;
        _remaining -= days * 86400;

        let hours: u64 = _remaining / 3600;
        _remaining -= hours * 3600;

        let minutes: u64 = _remaining / 60;
        _remaining -= minutes * 60;

        if months > 0 {
            time_string.push_str(&format!("{months} months "));
        }
        if weeks > 0 {
            time_string.push_str(&format!("{weeks} weeks "));
        }
        if days > 0 {
            time_string.push_str(&format!("{days} days "));
        }
        if hours > 0 {
            time_string.push_str(&format!("{hours} hours "));
        }
        if minutes > 0 {
            time_string.push_str(&format!("{minutes} minutes"));
        }

        time_string
    }
}

/// Represents information about memory usage.
#[derive(Deserialize, Serialize)]
pub struct MemInfo {
    used: u64,
    free: u64,
    total: u64,
    pretty: String,
}

impl MemInfo {
    /// Creates a new `MemInfo` instance based on total and free memory values.
    pub fn new(total: u64, free: u64) -> MemInfo {
        let used = total - free;
        let pretty = format!(
            "{}/{} ({})",
            ByteSize(total - free),
            ByteSize(total),
            ByteSize(free)
        );
        MemInfo {
            used,
            free,
            total,
            pretty,
        }
    }
}

/// Represents information about disk usage.
#[derive(Deserialize, Serialize)]
pub struct DiskInfo {
    mount_point: String,
    used: u64,
    free: u64,
    total: u64,
    pretty: String,
}

impl DiskInfo {
    /// Creates a new `DiskInfo` instance based on mount point, total, and free disk space values.
    pub fn new(mount_point: String, total: u64, free: u64) -> DiskInfo {
        let used = total - free;
        let pretty = format!(
            "{}: {}/{} ({})",
            mount_point,
            ByteSize(total - free),
            ByteSize(total),
            ByteSize(free)
        );
        DiskInfo {
            mount_point,
            used,
            free,
            total,
            pretty,
        }
    }
}

/// Represents hardware usage information including CPU load, memory usage, swap usage, disk info, and uptime.
#[derive(Deserialize, Serialize)]
pub struct HwUsage {
    cpu_load: (f32, f32, f32),
    memory_usage: MemInfo,
    swap_usage: MemInfo,
    disk_info: DiskInfo,
    uptime: UptimeInfo,
}

impl HwUsage {
    /// Creates a new `HwUsage` instance with information about CPU load, memory usage, swap usage, disk info, and uptime.
    pub fn new() -> HwUsage {
        let sys = systemstat::platform::linux::PlatformImpl::new();

        // TODO: needs some error handling
        let cpu_load = sys.load_average().unwrap();
        let memory_usage = sys.memory().unwrap();
        let swap_usage = sys.swap().unwrap();
        let uptime = sys.uptime().unwrap();
        let disk = sys.mount_at("/").unwrap(); // probably would need root previlage

        HwUsage {
            cpu_load: (cpu_load.one, cpu_load.five, cpu_load.fifteen),
            memory_usage: MemInfo::new(memory_usage.total.as_u64(), memory_usage.free.as_u64()),
            swap_usage: MemInfo::new(swap_usage.total.as_u64(), swap_usage.free.as_u64()),
            uptime: UptimeInfo::new(uptime.as_secs()),
            disk_info: DiskInfo::new(disk.fs_mounted_on, disk.total.as_u64(), disk.free.as_u64()),
        }
    }
}
