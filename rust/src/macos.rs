use crate::port::{KillResult, Signal};
use libproc::libproc::bsd_info::BSDInfo;
use libproc::libproc::file_info::{pidfdinfo, ListFDs, ProcFDType};
use libproc::libproc::net_info::{SocketFDInfo, SocketInfoKind};
use libproc::libproc::proc_pid::{listpidinfo, pidinfo};
use libproc::processes::{pids_by_type, ProcFilter};
use nix::sys::signal;
use nix::unistd::Pid;
use std::collections::HashSet;
use std::io;

pub fn find_and_kill(
    port: u16,
    protocol: &str,
    dry_run: bool,
    sig: Signal,
) -> Result<KillResult, io::Error> {
    if protocol != "tcp" && protocol != "udp" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported protocol: {protocol}"),
        ));
    }

    let pids = find_pids_on_port(port, protocol)?;

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

fn find_pids_on_port(port: u16, protocol: &str) -> Result<Vec<u32>, io::Error> {
    let mut matching_pids = HashSet::new();

    let all_pids = pids_by_type(ProcFilter::All)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for pid in all_pids {
        let pid_i32 = pid as i32;

        // Get process info to find number of open files
        let bsd_info = match pidinfo::<BSDInfo>(pid_i32, 0) {
            Ok(info) => info,
            Err(_) => continue,
        };

        // List file descriptors for this process
        let fds = match listpidinfo::<ListFDs>(pid_i32, bsd_info.pbi_nfiles as usize) {
            Ok(fds) => fds,
            Err(_) => continue, // Permission denied or process vanished
        };

        for fd in &fds {
            if ProcFDType::from(fd.proc_fdtype) as u32 != ProcFDType::Socket as u32 {
                continue;
            }

            let socket_info = match pidfdinfo::<SocketFDInfo>(pid_i32, fd.proc_fd) {
                Ok(info) => info,
                Err(_) => continue,
            };

            let local_port = match SocketInfoKind::from(socket_info.psi.soi_kind) {
                SocketInfoKind::Tcp => {
                    if protocol != "tcp" {
                        continue;
                    }
                    // Safety: soi_kind == Tcp so accessing pri_tcp is valid
                    let tcp_info = unsafe { socket_info.psi.soi_proto.pri_tcp };
                    let in_info = tcp_info.tcpsi_ini;
                    let lport = in_info.insi_lport;
                    u16::from_be(lport as u16)
                }
                SocketInfoKind::In => {
                    if protocol != "udp" {
                        continue;
                    }
                    // Safety: soi_kind == In so accessing pri_in is valid
                    let in_sockinfo = unsafe { socket_info.psi.soi_proto.pri_in };
                    let lport = in_sockinfo.insi_lport;
                    u16::from_be(lport as u16)
                }
                _ => continue,
            };

            if local_port == port {
                matching_pids.insert(pid);
                break; // Found a match for this PID
            }
        }
    }

    Ok(matching_pids.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pids_unused_port() {
        let pids = find_pids_on_port(19999, "tcp").unwrap();
        assert!(pids.is_empty());
    }

    #[test]
    fn test_invalid_protocol() {
        let result = find_and_kill(3000, "sctp", true, Signal::Kill);
        assert!(result.is_err());
    }

    #[test]
    fn test_dry_run_does_not_kill() {
        let result = find_and_kill(19999, "tcp", true, Signal::Kill).unwrap();
        assert!(result.pids.is_empty());
        assert!(result.error.is_none());
    }
}
