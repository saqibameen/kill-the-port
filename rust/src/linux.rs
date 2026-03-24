use crate::port::{KillResult, Signal};
use nix::sys::signal;
use nix::unistd::Pid;
use std::collections::HashSet;
use std::fs;
use std::io;

pub fn find_and_kill(
    port: u16,
    protocol: &str,
    dry_run: bool,
    sig: Signal,
) -> Result<KillResult, io::Error> {
    let inodes = find_socket_inodes(port, protocol)?;
    if inodes.is_empty() {
        return Ok(KillResult {
            port,
            pids: vec![],
            error: None,
        });
    }

    let pids = find_pids_for_inodes(&inodes)?;
    if pids.is_empty() {
        return Ok(KillResult {
            port,
            pids: vec![],
            error: None,
        });
    }

    if !dry_run {
        let nix_signal = match sig {
            Signal::Kill => signal::Signal::SIGKILL,
            Signal::Term => signal::Signal::SIGTERM,
        };
        for &pid in &pids {
            let _ = signal::kill(Pid::from_raw(pid as i32), nix_signal);
        }
    }

    Ok(KillResult {
        port,
        pids,
        error: None,
    })
}

/// Read /proc/net/{tcp,tcp6,udp,udp6} to find socket inodes matching the port
fn find_socket_inodes(port: u16, protocol: &str) -> Result<HashSet<u64>, io::Error> {
    let mut inodes = HashSet::new();
    let files = match protocol {
        "tcp" => vec!["/proc/net/tcp", "/proc/net/tcp6"],
        "udp" => vec!["/proc/net/udp", "/proc/net/udp6"],
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported protocol: {protocol}"),
            ))
        }
    };

    for path in files {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // File might not exist (e.g. no IPv6)
        };

        for line in content.lines().skip(1) {
            // Each line: sl local_address rem_address st ...
            // local_address is hex_ip:hex_port
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            let local_addr = parts[1];
            if let Some(hex_port) = local_addr.split(':').nth(1) {
                if let Ok(p) = u16::from_str_radix(hex_port, 16) {
                    if p == port {
                        // For TCP, only match LISTEN state (0A)
                        if protocol == "tcp" && parts[3] != "0A" {
                            continue;
                        }
                        if let Ok(inode) = parts[9].parse::<u64>() {
                            inodes.insert(inode);
                        }
                    }
                }
            }
        }
    }

    Ok(inodes)
}

/// Scan /proc/[pid]/fd/ to find which PIDs hold the target socket inodes
fn find_pids_for_inodes(inodes: &HashSet<u64>) -> Result<Vec<u32>, io::Error> {
    let mut pids = HashSet::new();

    let proc_entries = match fs::read_dir("/proc") {
        Ok(e) => e,
        Err(e) => return Err(e),
    };

    for entry in proc_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Only look at numeric directories (PIDs)
        let pid: u32 = match name_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_dir = format!("/proc/{pid}/fd");
        let fds = match fs::read_dir(&fd_dir) {
            Ok(f) => f,
            Err(_) => continue, // Permission denied or process vanished
        };

        for fd_entry in fds {
            let fd_entry = match fd_entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let link = match fs::read_link(fd_entry.path()) {
                Ok(l) => l,
                Err(_) => continue,
            };

            let link_str = link.to_string_lossy();
            // Socket links look like: socket:[12345]
            if let Some(inode_str) = link_str.strip_prefix("socket:[").and_then(|s| s.strip_suffix(']')) {
                if let Ok(inode) = inode_str.parse::<u64>() {
                    if inodes.contains(&inode) {
                        pids.insert(pid);
                        break; // Found match for this PID, move on
                    }
                }
            }
        }
    }

    Ok(pids.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_socket_inodes_invalid_protocol() {
        let result = find_socket_inodes(3000, "sctp");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_pids_empty_inodes() {
        let inodes = HashSet::new();
        let pids = find_pids_for_inodes(&inodes).unwrap();
        assert!(pids.is_empty());
    }

    #[test]
    fn test_find_and_kill_unused_port() {
        // Port 19999 should not have anything running
        let result = find_and_kill(19999, "tcp", true, Signal::Kill).unwrap();
        assert!(result.pids.is_empty());
        assert!(result.error.is_none());
    }
}
