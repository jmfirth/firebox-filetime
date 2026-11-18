use crate::FileTime;
use std::fs::{self, File};
use std::io;
use std::path::Path;

pub fn set_file_times(path: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    set_times(path, Some(atime), Some(mtime), false)
}

pub fn set_symlink_file_times(path: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    set_times(path, Some(atime), Some(mtime), true)
}

pub fn set_file_mtime(path: &Path, mtime: FileTime) -> io::Result<()> {
    set_times(path, None, Some(mtime), false)
}

pub fn set_file_atime(path: &Path, atime: FileTime) -> io::Result<()> {
    set_times(path, Some(atime), None, false)
}

#[cfg(all(target_os = "wasi", target_vendor = "wasmer"))]
fn timespec(time: Option<FileTime>) -> libc::timespec {
    let mut ts: libc::timespec = unsafe { std::mem::zeroed() };
    if let Some(time) = time {
        ts.tv_sec = time.unix_seconds() as _;
        ts.tv_nsec = time.nanoseconds() as _;
    } else {
        ts.tv_nsec = libc::UTIME_OMIT;
    }
    ts
}

#[cfg(all(target_os = "wasi", target_vendor = "wasmer"))]
fn set_times(path: &Path, atime: Option<FileTime>, mtime: Option<FileTime>, nofollow: bool) -> io::Result<()> {
    use std::ffi::CString;
    let path = CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let times = [timespec(atime), timespec(mtime)];
    let flags = if nofollow { libc::AT_SYMLINK_NOFOLLOW } else { 0 };
    if unsafe { libc::utimensat(libc::AT_FDCWD, path.as_ptr(), times.as_ptr(), flags) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(all(target_os = "wasi", target_vendor = "wasmer")))]
fn set_times(_: &Path, _: Option<FileTime>, _: Option<FileTime>, _: bool) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    FileTime::from_system_time(meta.modified().expect("WASIX metadata::modified"))
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    FileTime::from_system_time(meta.accessed().expect("WASIX metadata::accessed"))
}

pub fn from_creation_time(meta: &fs::Metadata) -> Option<FileTime> {
    meta.created().ok().map(FileTime::from_system_time)
}

#[cfg(all(target_os = "wasi", target_vendor = "wasmer"))]
pub fn set_file_handle_times(
    file: &File,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    let times = [timespec(atime), timespec(mtime)];
    if unsafe { libc::futimens(file.as_raw_fd(), times.as_ptr()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(all(target_os = "wasi", target_vendor = "wasmer")))]
pub fn set_file_handle_times(_: &File, _: Option<FileTime>, _: Option<FileTime>) -> io::Result<()> {
    Err(io::ErrorKind::Unsupported.into())
}
