//! Nonblocking leases use Android flock because std file locks are unsupported there.

use std::{fs::File, io};

pub(crate) fn shared(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return android_lock(file, libc::LOCK_SH | libc::LOCK_NB);
    #[cfg(not(target_os = "android"))]
    file.try_lock_shared().map_err(Into::into)
}

pub(crate) fn exclusive(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return android_lock(file, libc::LOCK_EX | libc::LOCK_NB);
    #[cfg(not(target_os = "android"))]
    file.try_lock().map_err(Into::into)
}

pub(crate) fn unlock(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return android_lock(file, libc::LOCK_UN);
    #[cfg(not(target_os = "android"))]
    file.unlock()
}

#[cfg(target_os = "android")]
fn android_lock(file: &File, operation: libc::c_int) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    // SAFETY: File owns a live descriptor; flock neither retains nor closes it.
    if unsafe { libc::flock(file.as_raw_fd(), operation) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
