use crate::port::{KillResult, Signal};
use std::io;
use std::mem;
use std::ptr;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID,
    MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, MIB_UDP6ROW_OWNER_PID,
    MIB_UDP6TABLE_OWNER_PID, MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID,
    TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, TerminateProcess, PROCESS_TERMINATE,
};

pub fn find_and_kill(
    port: u16,
    protocol: &str,
    dry_run: bool,
    _sig: Signal, // Windows always uses TerminateProcess (equivalent to SIGKILL)
) -> Result<KillResult, io::Error> {
    let pids = match protocol {
        "tcp" => find_tcp_pids(port)?,
        "udp" => find_udp_pids(port)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported protocol: {protocol}"),
            ))
        }
    };

    if !dry_run {
        for &pid in &pids {
            kill_process(pid)?;
        }
    }

    Ok(KillResult {
        port,
        pids,
        error: None,
    })
}

fn find_tcp_pids(port: u16) -> Result<Vec<u32>, io::Error> {
    let mut pids = std::collections::HashSet::new();

    // IPv4
    let mut size: u32 = 0;
    unsafe {
        GetExtendedTcpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
    }

    if size > 0 {
        let mut buf = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedTcpTable(
                buf.as_mut_ptr() as *mut _,
                &mut size,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        };
        if ret == 0 {
            let table = unsafe { &*(buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID) };
            let rows = unsafe {
                std::slice::from_raw_parts(
                    table.table.as_ptr(),
                    table.dwNumEntries as usize,
                )
            };
            for row in rows {
                let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
                if local_port == port {
                    pids.insert(row.dwOwningPid);
                }
            }
        }
    }

    // IPv6
    size = 0;
    unsafe {
        GetExtendedTcpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
    }

    if size > 0 {
        let mut buf = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedTcpTable(
                buf.as_mut_ptr() as *mut _,
                &mut size,
                0,
                AF_INET6 as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        };
        if ret == 0 {
            let table = unsafe { &*(buf.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID) };
            let rows = unsafe {
                std::slice::from_raw_parts(
                    table.table.as_ptr(),
                    table.dwNumEntries as usize,
                )
            };
            for row in rows {
                let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
                if local_port == port {
                    pids.insert(row.dwOwningPid);
                }
            }
        }
    }

    Ok(pids.into_iter().collect())
}

fn find_udp_pids(port: u16) -> Result<Vec<u32>, io::Error> {
    let mut pids = std::collections::HashSet::new();

    // IPv4
    let mut size: u32 = 0;
    unsafe {
        GetExtendedUdpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );
    }

    if size > 0 {
        let mut buf = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedUdpTable(
                buf.as_mut_ptr() as *mut _,
                &mut size,
                0,
                AF_INET as u32,
                UDP_TABLE_OWNER_PID,
                0,
            )
        };
        if ret == 0 {
            let table = unsafe { &*(buf.as_ptr() as *const MIB_UDPTABLE_OWNER_PID) };
            let rows = unsafe {
                std::slice::from_raw_parts(
                    table.table.as_ptr(),
                    table.dwNumEntries as usize,
                )
            };
            for row in rows {
                let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
                if local_port == port {
                    pids.insert(row.dwOwningPid);
                }
            }
        }
    }

    // IPv6
    size = 0;
    unsafe {
        GetExtendedUdpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET6 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );
    }

    if size > 0 {
        let mut buf = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedUdpTable(
                buf.as_mut_ptr() as *mut _,
                &mut size,
                0,
                AF_INET6 as u32,
                UDP_TABLE_OWNER_PID,
                0,
            )
        };
        if ret == 0 {
            let table = unsafe { &*(buf.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID) };
            let rows = unsafe {
                std::slice::from_raw_parts(
                    table.table.as_ptr(),
                    table.dwNumEntries as usize,
                )
            };
            for row in rows {
                let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
                if local_port == port {
                    pids.insert(row.dwOwningPid);
                }
            }
        }
    }

    Ok(pids.into_iter().collect())
}

fn kill_process(pid: u32) -> Result<(), io::Error> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let result = TerminateProcess(handle, 1);
        CloseHandle(handle);
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
