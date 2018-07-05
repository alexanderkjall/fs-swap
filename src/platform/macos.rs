extern crate libc;
extern crate libloading;

use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{io, ffi};
use self::libloading::os::unix::Library;

unsafe fn get_renamex_np() -> Option<libloading::os::unix::Symbol<unsafe extern fn (oldpath: *const libc::c_char, newpath: *const libc::c_char, flags: libc::c_uint) -> libc::c_int>> {
	let lib = Library::this();
	lib.get(b"renamex_np").ok()
}

extern "stdcall" {
	fn exchangedata(oldpath: *const libc::c_char, newpath: *const libc::c_char, flags: libc::c_uint) -> libc::c_int;
}

pub fn swap<A, B>(a: A, b: B) -> io::Result<()> where A: AsRef<Path>, B: AsRef<Path> {
	const RENAME_SWAP: libc::c_uint = 2;

	let a_path = ffi::CString::new(a.as_ref().as_os_str().as_bytes())?;
	let b_path = ffi::CString::new(b.as_ref().as_os_str().as_bytes())?;

	unsafe {
		if let Some(renamex_np) = get_renamex_np() {
			if renamex_np(a_path.as_ptr(), b_path.as_ptr(), RENAME_SWAP) == 0 {
				return Ok(())
			}

			let err = *libc::__error();
			if err != libc::ENOTSUP {
				return Err(io::Error::new(io::ErrorKind::Other, format!("renamex_np failed with code: {}", err)));
			}
		}

		// some volumes do not support swapping
		// it these cases, let's try to swap files using
		// swapping directories returns `EINVAL`
		if exchangedata(a_path.as_ptr(), b_path.as_ptr(), 0) == 0 {
			return Ok(())
		}

		Err(io::Error::new(io::ErrorKind::Other, format!("exchangedata failed with code: {}", *libc::__error())))
	}
}
