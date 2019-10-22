use std::mem;
use libc::{c_int, flock};

#[no_mangle]
pub extern "C" fn c_lock(fd: c_int, is_blocking: c_int, is_writable: c_int) -> c_int {
  if fd < 0 {
    return libc::EBADF;
  }

  let mut fl: flock = unsafe { mem::zeroed() };

  fl.l_type = if is_writable != 0 { libc::F_WRLCK } else { libc::F_RDLCK } as libc::c_short;
  fl.l_whence = libc::SEEK_SET as libc::c_short;
  fl.l_start = 0;
  fl.l_len = 0;

  let result = unsafe {
    libc::fcntl(fd, if is_blocking != 0 { libc::F_SETLKW } else { libc::F_SETLK }, &fl)
  };

  if result == -1 {
    return unsafe { *libc::__errno_location() };
  }

  return 0;
}

#[no_mangle]
pub extern "C" fn c_unlock(fd: c_int) -> c_int {
  if fd < 0 {
    return libc::EBADF;
  }

  let mut fl: flock = unsafe { mem::zeroed() };
  fl.l_type   = libc::F_UNLCK as libc::c_short;
  fl.l_whence = libc::SEEK_SET as libc::c_short;
  fl.l_start  = 0;
  fl.l_len    = 0;

  let result = unsafe {
    libc::fcntl(fd, libc::F_SETLK, &mut fl)
  };

  if result == -1 {
    return unsafe { *libc::__errno_location() };
  }

  return 0;
}
