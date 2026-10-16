//! Android uses Bionic flock because std file locking is unsupported on that target.

use std::{fs::File, io};

pub(crate) fn shared(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return flock(file, libc::LOCK_SH | libc::LOCK_NB);
    #[cfg(not(target_os = "android"))]
    file.try_lock_shared().map_err(io::Error::from)
}

pub(crate) fn exclusive(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return flock(file, libc::LOCK_EX | libc::LOCK_NB);
    #[cfg(not(target_os = "android"))]
    file.try_lock().map_err(io::Error::from)
}

pub(crate) fn unlock(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    return flock(file, libc::LOCK_UN);
    #[cfg(not(target_os = "android"))]
    file.unlock()
}

#[cfg(target_os = "android")]
fn flock(file: &File, operation: libc::c_int) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    // The borrowed File keeps the descriptor alive; flock neither owns nor closes it.
    if unsafe { libc::flock(file.as_raw_fd(), operation) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
