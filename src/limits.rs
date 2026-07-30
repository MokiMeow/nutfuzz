use std::process::Command;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceLimits {
    pub address_space_bytes: Option<u64>,
    pub cpu_seconds: Option<u64>,
    pub file_size_bytes: Option<u64>,
}

impl ResourceLimits {
    pub fn is_empty(self) -> bool {
        self.address_space_bytes.is_none()
            && self.cpu_seconds.is_none()
            && self.file_size_bytes.is_none()
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn configure(
    command: &mut Command,
    limits: ResourceLimits,
) -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(move || {
            set_limit(RLIMIT_CORE, Some(0))?;
            set_limit(RLIMIT_AS, limits.address_space_bytes)?;
            set_limit(RLIMIT_CPU, limits.cpu_seconds)?;
            set_limit(RLIMIT_FSIZE, limits.file_size_bytes)
        });
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn configure(
    _command: &mut Command,
    limits: ResourceLimits,
) -> Result<(), String> {
    if limits.is_empty() {
        Ok(())
    } else {
        Err("resource limits are supported only on Linux".into())
    }
}

#[cfg(target_os = "linux")]
fn set_limit(resource: i32, value: Option<u64>) -> std::io::Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let limit = RLimit {
        current: value,
        maximum: value,
    };
    if unsafe { setrlimit(resource, &limit) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct RLimit {
    current: u64,
    maximum: u64,
}

#[cfg(target_os = "linux")]
const RLIMIT_CPU: i32 = 0;
#[cfg(target_os = "linux")]
const RLIMIT_FSIZE: i32 = 1;
#[cfg(target_os = "linux")]
const RLIMIT_CORE: i32 = 4;
#[cfg(target_os = "linux")]
const RLIMIT_AS: i32 = 9;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn setrlimit(resource: i32, limit: *const RLimit) -> i32;
}
