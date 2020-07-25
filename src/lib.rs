//! File locking via POSIX advisory record locks.
//!
//! This crate provides the facility to obtain a write-lock and unlock a file
//! following the advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001
//! (POSIX.1) via `fcntl()`.
//!
//! # Examples
//!
//! Please note that the examples use `tempfile` merely to quickly create a file
//! which is removed automatically. In the common case, you would want to lock
//! a file which is known to multiple processes.
//!
//! ```
//! use file_locker::FileLock;
//! use std::io::prelude::*;
//! use std::io::Result;
//!
//! fn main() -> Result<()> {
//!     let mut filelock = FileLock::new("myfile.txt")
//!                         .blocking(true)
//!                         .writeable(true)
//!                         .lock()?;
//!
//!     filelock.file.write_all(b"Hello, World!")?;
//!
//!     // Manually unlocking is optional as we unlock on Drop
//!     filelock.unlock()?;
//!     Ok(())
//! }
//! ```

use nix::{
    fcntl::{fcntl, FcntlArg},
    libc,
};
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, Error, ErrorKind, IoSlice, IoSliceMut, Result, SeekFrom},
    os::unix::{
        fs::FileExt,
        io::{AsRawFd, RawFd},
    },
    path::Path,
};

/// Represents the actually locked file
#[derive(Debug)]
pub struct FileLock {
    /// the `std::fs::File` of the file that's locked
    pub file: File,
}

impl FileLock {
    /// Create a [`FileLockBuilder`](struct.FileLockBuilder.html)
    ///
    /// blocking and writeable default to false
    ///
    /// # Examples
    ///
    ///```
    ///use file_locker::FileLock;
    ///use std::io::prelude::*;
    ///use std::io::Result;
    ///
    ///fn main() -> Result<()> {
    ///    let mut filelock = FileLock::new("myfile.txt")
    ///                     .writeable(true)
    ///                     .blocking(true)
    ///                     .lock()?;
    ///
    ///    filelock.file.write_all(b"Hello, world")?;
    ///    Ok(())
    ///}
    ///```
    ///
    pub fn new<T: AsRef<Path>>(file_path: T) -> FileLockBuilder<T> {
        FileLockBuilder {
            file_path,
            blocking: false,
            writeable: false,
        }
    }

    /// Try to lock the specified file
    ///
    /// # Parameters
    ///
    /// - `filename` is the path of the file we want to lock on
    ///
    /// - `is_blocking` is a flag to indicate if we should block if it's already locked
    ///
    /// If set, this call will block until the lock can be obtained.  
    /// If not set, this call will return immediately, giving an error if it would block
    ///
    /// - `is_writable` is a flag to indicate if we want to lock for writing
    ///
    /// # Examples
    ///
    ///```
    ///use file_locker::FileLock;
    ///use std::io::prelude::*;
    ///use std::io::Result;
    ///
    ///fn main() -> Result<()> {
    ///    let mut filelock = FileLock::lock("myfile.txt", false, false)?;
    ///
    ///    let mut buf = String::new();
    ///    filelock.file.read_to_string(&mut buf)?;
    ///    Ok(())
    ///}
    ///```
    ///
    pub fn lock(
        file_path: impl AsRef<Path>,
        blocking: bool,
        writeable: bool,
    ) -> Result<FileLock> {
        let file = OpenOptions::new()
            .read(true)
            .write(writeable)
            .create(writeable)
            .open(&file_path)?;
        let flock = libc::flock {
            l_type: if writeable {
                libc::F_WRLCK
            } else {
                libc::F_RDLCK
            } as i16,
            l_whence: libc::SEEK_SET as i16,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };
        let arg = if blocking {
            FcntlArg::F_SETLKW(&flock)
        } else {
            FcntlArg::F_SETLK(&flock)
        };
        fcntl(file.as_raw_fd(), arg).map_err(cver)?;
        Ok(Self { file })
    }

    /// Unlock our locked file
    ///
    /// *Note:* This method is optional as the file lock will be unlocked automatically when dropped
    ///
    /// # Examples
    ///
    ///```
    ///use file_locker::FileLock;
    ///use std::io::prelude::*;
    ///use std::io::Result;
    ///
    ///fn main() -> Result<()> {
    ///    let mut filelock = FileLock::new("myfile.txt")
    ///                     .writeable(true)
    ///                     .blocking(true)
    ///                     .lock()?;
    ///
    ///    filelock.file.write_all(b"Hello, world")?;
    ///
    ///    filelock.unlock()?;
    ///    Ok(())
    ///}
    ///```
    ///
    pub fn unlock(&self) -> Result<()> {
        let flock = libc::flock {
            l_type: libc::F_UNLCK as i16,
            l_whence: libc::SEEK_SET as i16,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        };
        fcntl(self.file.as_raw_fd(), FcntlArg::F_SETLK(&flock))
            .map_err(cver)?;
        Ok(())
    }
}

impl Read for FileLock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> Result<usize> {
        self.file.read_vectored(bufs)
    }
}

impl Write for FileLock {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }

    fn write_vectored(&mut self, bufs: &[IoSlice]) -> Result<usize> {
        self.file.write_vectored(bufs)
    }
}

impl Seek for FileLock {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.file.seek(pos)
    }
}

impl AsRawFd for FileLock {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl FileExt for FileLock {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.file.read_at(buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        self.file.write_at(buf, offset)
    }
}

/// Builder to create [`FileLock`](struct.FileLock.html)
///
/// blocking and writeable default to false
#[derive(Debug)]
pub struct FileLockBuilder<T> {
    file_path: T,
    blocking: bool,
    writeable: bool,
}

impl<T: AsRef<Path>> FileLockBuilder<T> {
    /// Set lock to blocking mode
    pub fn blocking(mut self, v: bool) -> Self {
        self.blocking = v;
        self
    }

    /// Open file as writeable and get exclusive lock
    pub fn writeable(mut self, v: bool) -> Self {
        self.writeable = v;
        self
    }

    /// Create a [`FileLock`](struct.FileLock.html) with these parameters.
    /// Calls [`FileLock::lock`](struct.FileLock.html#method.lock)
    pub fn lock(self) -> Result<FileLock> {
        FileLock::lock(self.file_path, self.blocking, self.writeable)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

fn cver(e: nix::Error) -> Error {
    match e.as_errno() {
        Some(e) => Error::from_raw_os_error(e as i32),
        None => Error::new(ErrorKind::Other, e),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use nix::unistd::fork;
    use nix::unistd::ForkResult::{Child, Parent};
    use std::fs::remove_file;
    use std::process;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn lock_and_unlock() {
        let filename = "filelock.test";

        for already_exists in &[true, false] {
            for already_locked in &[true, false] {
                for already_writable in &[true, false] {
                    for is_blocking in &[true, false] {
                        for is_writable in &[true, false] {
                            if !*already_exists
                                && (*already_locked || *already_writable)
                            {
                                // nonsensical tests
                                continue;
                            }

                            let _ = remove_file(&filename);

                            let parent_lock = match *already_exists {
                                false => None,
                                true => {
                                    let _ = OpenOptions::new()
                                        .write(true)
                                        .create(true)
                                        .open(&filename);

                                    match *already_locked {
                                        false => None,
                                        true => {
                                            match FileLock::lock(&filename, true, *already_writable)
                                            {
                                                Ok(lock) => Some(lock),
                                                Err(err) => {
                                                    panic!("Error creating parent lock ({})", err)
                                                }
                                            }
                                        }
                                    }
                                }
                            };

                            match fork() {
                                Ok(Parent { child: _ }) => {
                                    sleep(Duration::from_millis(150));

                                    match parent_lock {
                                        Some(lock) => {
                                            let _ = lock.unlock();
                                        }
                                        None => {}
                                    }

                                    sleep(Duration::from_millis(350));
                                }
                                Ok(Child) => {
                                    let mut try_count = 0;
                                    let mut locked = false;

                                    match *already_locked {
                                        true => match *is_blocking {
                                            true => {
                                                match FileLock::lock(filename, *is_blocking, *is_writable) {
                                                    Ok(_)  => { locked = true },
                                                    Err(_) => panic!("Error getting lock after wating for release"),
                                                }
                                            }
                                            false => {
                                                for _ in 0..5 {
                                                    match FileLock::lock(
                                                        filename,
                                                        *is_blocking,
                                                        *is_writable,
                                                    ) {
                                                        Ok(_) => {
                                                            locked = true;
                                                            break;
                                                        }
                                                        Err(_) => {
                                                            sleep(Duration::from_millis(50));
                                                            try_count = try_count + 1;
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        false => match FileLock::lock(
                                            filename,
                                            *is_blocking,
                                            *is_writable,
                                        ) {
                                            Ok(_) => locked = true,
                                            Err(_) => match !*already_exists && !*is_writable {
                                                true => {}
                                                false => {
                                                    panic!("Error getting lock with no competition")
                                                }
                                            },
                                        },
                                    }

                                    match !*already_exists && !is_writable {
                                        true => assert!(
                                            locked == false,
                                            "Locking a non-existent file for reading should fail"
                                        ),
                                        false => assert!(
                                            locked == true,
                                            "Lock should have been successful"
                                        ),
                                    }

                                    match *is_blocking {
                                        true  => assert!(try_count == 0, "Try count should be zero when blocking"),
                                        false => {
                                            match *already_locked {
                                                false => assert!(try_count == 0, "Try count should be zero when no competition"),
                                                true  => match !*already_writable && !is_writable {
                                                    true  => assert!(try_count == 0, "Read lock when locked for reading should succeed first go"),
                                                    false => assert!(try_count >= 3, "Try count should be >= 3"),
                                                },
                                            }
                                        },
                                    }

                                    process::exit(7);
                                }
                                Err(_) => {
                                    panic!("Error forking tests :(");
                                }
                            }

                            let _ = remove_file(&filename);
                        }
                    }
                }
            }
        }
    }
}
