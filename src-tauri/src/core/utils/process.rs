use std::{collections::HashSet, path::PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub process_name: String,
    pub pid: u32,
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindProcessResult {
    pub running: bool,
    pub process: Option<ProcessInfo>,
}

fn normalize_process_identity(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.strip_suffix(".exe").unwrap_or(&normalized).to_string()
}

fn process_matches_candidate(
    process_name: &str,
    executable_path: &Option<PathBuf>,
    candidate: &str,
) -> bool {
    let candidate = normalize_process_identity(candidate);

    if normalize_process_identity(process_name) == candidate {
        return true;
    }

    let Some(executable_path) = executable_path.as_ref() else {
        return false;
    };

    if let Some(file_name) = executable_path.file_name() {
        if normalize_process_identity(&file_name.to_string_lossy()) == candidate {
            return true;
        }
    }

    if let Some(file_stem) = executable_path.file_stem() {
        if normalize_process_identity(&file_stem.to_string_lossy()) == candidate {
            return true;
        }
    }

    false
}

#[cfg(target_os = "windows")]
pub mod finder {
    use super::*;

    /// 杀死指定进程
    pub fn kill_process(pid: u32) -> Result<(), String> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

        unsafe {
            let process_handle =
                OpenProcess(PROCESS_TERMINATE, false, pid).map_err(|e| e.to_string())?;

            TerminateProcess(process_handle, 0).map_err(|e| e.to_string())?;
            CloseHandle(process_handle).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// 查找指定进程的 PID 和路径
    fn find_process_sysinfo(process_pattern: &[String]) -> HashSet<(String, u32, Option<PathBuf>)> {
        use sysinfo::System;

        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        sys.processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name().to_string_lossy().to_string().to_lowercase();
                process_pattern.iter().any(|pattern| {
                    let pattern_lower = pattern.trim().to_ascii_lowercase();
                    !pattern_lower.is_empty() && name.contains(&pattern_lower)
                })
            })
            .map(|(pid, process)| {
                let process_name = process.name().to_string_lossy().to_string();
                let process_path = process.exe().map(|v| v.to_path_buf());

                (process_name, pid.as_u32(), process_path)
            })
            .collect::<HashSet<(String, u32, Option<PathBuf>)>>()
    }

    // 获取进程路径
    fn get_process_path_by_pid(pid: u32) -> Option<PathBuf> {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows::Win32::Foundation::{CloseHandle, GetLastError, HMODULE, MAX_PATH};
        use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        };

        unsafe {
            // 以查询信息和读取内存的权限打开进程
            let process_handle = if let Ok(process_handle) =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
            {
                process_handle
            } else {
                log::debug!("无法打开进程 (PID: {}): {:?}", pid, GetLastError());
                return None;
            };

            if process_handle.is_invalid() {
                log::debug!("无法打开进程 (PID: {}): {:?}", pid, GetLastError());
                return None;
            }

            // 创建一个缓冲区来接收路径
            let mut path_buffer: Vec<u16> = vec![0; MAX_PATH as usize];
            let path_len = GetModuleFileNameExW(
                Some(process_handle),
                Some(HMODULE::default()),
                &mut path_buffer,
            );

            // 关闭进程句柄
            let _ = CloseHandle(process_handle); // 忽略关闭结果

            if path_len == 0 {
                log::debug!("无法获取进程路径 (PID: {}): {:?}", pid, GetLastError());
                return None;
            }

            // 调整缓冲区大小为实际长度
            path_buffer.truncate(path_len as usize);

            // 将 UTF-16 缓冲区转换为 OsString，然后是 PathBuf
            Some(PathBuf::from(OsString::from_wide(&path_buffer)))
        }
    }

    /// 查找进程， 返回进程的 PID 和路径
    pub fn find_process(process_pattern: &[String]) -> HashSet<(String, u32, Option<PathBuf>)> {
        let processes = find_process_sysinfo(process_pattern);
        let processes = processes
            .iter()
            .map(|(process_name, pid, process_path_buf)| {
                let process_path_buf =
                    process_path_buf.clone().or_else(|| get_process_path_by_pid(*pid));
                (process_name.clone(), *pid, process_path_buf)
            })
            .collect::<HashSet<(String, u32, Option<PathBuf>)>>();

        processes
    }

    /// 测试杀手指定名称的进程
    #[test]
    fn test_kill_process() {
        let process_patterns = vec!["brave".to_string()];
        let processes = find_process(&process_patterns);
        for (_, pid, _) in processes {
            let _ = kill_process(pid);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod finder {
    use super::*;
    use sysinfo::{Pid, System};

    pub fn find_process(process_pattern: &[String]) -> HashSet<(String, u32, Option<PathBuf>)> {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        sys.processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name().to_string_lossy().to_string().to_lowercase();
                process_pattern.iter().any(|pattern| {
                    let pattern_lower = pattern.trim().to_ascii_lowercase();
                    !pattern_lower.is_empty() && name.contains(&pattern_lower)
                })
            })
            .map(|(pid, process)| {
                let process_name = process.name().to_string_lossy().to_string();
                let process_path = process.exe().map(|v| v.to_path_buf());

                (process_name, pid.as_u32(), process_path)
            })
            .collect::<HashSet<(String, u32, Option<PathBuf>)>>()
    }

    /// 杀死指定进程
    pub fn kill_process(pid: u32) -> Result<(), String> {
        let sys = System::new();

        if let Some(process) = sys.process(Pid::from_u32(pid)) {
            process.kill();
            Ok(())
        } else {
            Err(format!("进程不存在: {}", pid))
        }
    }
}

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return finder::kill_process(pid);
    #[cfg(not(target_os = "windows"))]
    return finder::kill_process(pid);
}

#[tauri::command]
pub fn find_process(
    names: Vec<String>,
    match_all: Option<bool>,
) -> Result<FindProcessResult, String> {
    let normalized_names = names
        .into_iter()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    let require_all = match_all.unwrap_or(false);

    if normalized_names.is_empty() {
        return Ok(FindProcessResult {
            running: false,
            process: None,
        });
    }

    let matches = finder::find_process(&normalized_names);

    let matched_processes = normalized_names
        .iter()
        .filter_map(|candidate| {
            matches
                .iter()
                .find(|(process_name, _, executable_path)| {
                    process_matches_candidate(process_name, executable_path, candidate)
                })
                .map(|(process_name, pid, executable_path)| ProcessInfo {
                    process_name: process_name.clone(),
                    pid: *pid,
                    executable_path: executable_path
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string()),
                })
        })
        .collect::<Vec<_>>();

    if require_all {
        return Ok(FindProcessResult {
            running: matched_processes.len() == normalized_names.len(),
            process: matched_processes.into_iter().next(),
        });
    }

    if let Some(process) = matched_processes.into_iter().next() {
        return Ok(FindProcessResult {
            running: true,
            process: Some(process),
        });
    }

    Ok(FindProcessResult {
        running: false,
        process: None,
    })
}
