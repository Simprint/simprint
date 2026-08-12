use crate::app::{Result, RuntimeError};
#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::Arc;
#[cfg(target_os = "windows")]
use tokio::sync::RwLock;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    },
    System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE},
};

#[cfg(target_os = "windows")]
pub struct JobHandle {
    handle: HANDLE,
}

#[cfg(target_os = "windows")]
impl JobHandle {
    pub fn create() -> Result<Self> {
        unsafe {
            let handle = CreateJobObjectW(None, None).map_err(|error| {
                RuntimeError::Internal(format!("Failed to create job object: {}", error))
            })?;

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|error| {
                RuntimeError::Internal(format!("Failed to set job object information: {}", error))
            })?;

            Ok(Self { handle })
        }
    }

    pub fn assign_process(&self, pid: u32) -> Result<()> {
        unsafe {
            let process_handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid)
                .map_err(|error| {
                    RuntimeError::Internal(format!("Failed to open process {}: {}", pid, error))
                })?;

            AssignProcessToJobObject(self.handle, process_handle).map_err(|error| {
                RuntimeError::Internal(format!("Failed to assign process to job: {}", error))
            })?;

            let _ = CloseHandle(process_handle);
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for JobHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(target_os = "windows")]
unsafe impl Send for JobHandle {}
#[cfg(target_os = "windows")]
unsafe impl Sync for JobHandle {}

pub struct JobManager {
    #[cfg(target_os = "windows")]
    jobs: Arc<RwLock<HashMap<String, Arc<JobHandle>>>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[cfg(target_os = "windows")]
    pub async fn create_and_assign(&self, env_uuid: &str, pid: u32) -> Result<()> {
        let job = JobHandle::create()?;
        job.assign_process(pid)?;

        let mut jobs = self.jobs.write().await;
        jobs.insert(env_uuid.to_string(), Arc::new(job));
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn create_and_assign(&self, _env_uuid: &str, _pid: u32) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn remove(&self, env_uuid: &str) {
        let mut jobs = self.jobs.write().await;
        jobs.remove(env_uuid);
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn remove(&self, _env_uuid: &str) {}

    #[cfg(target_os = "windows")]
    pub async fn clear_all(&self) {
        let mut jobs = self.jobs.write().await;
        jobs.clear();
    }

    #[cfg(not(target_os = "windows"))]
    pub async fn clear_all(&self) {}
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}
