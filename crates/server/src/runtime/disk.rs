//! Filesystem capacity probing.
//!
//! The server refuses writes when the data directory runs low on space, which means asking the
//! OS how much is left. There is no safe wrapper for `statvfs` in the dependency set, so this is
//! the one place in `piramid-server` that uses `unsafe` — factored here so both the readiness
//! check and the admin metrics endpoint share a single audited implementation.

use piramid_core::error::{Result, ServerError};

/// Total and available bytes on the filesystem backing `path`.
///
/// Returns `(None, None)` on non-Unix targets, where `statvfs` does not exist.
#[cfg_attr(not(target_family = "unix"), allow(unused_variables))]
pub fn stats(path: &str) -> Result<(Option<u64>, Option<u64>)> {
    #[cfg(target_family = "unix")]
    {
        use std::ffi::CString;

        let c_path = CString::new(path)
            .map_err(|_| ServerError::Internal("data_dir contains an interior NUL byte".into()))?;

        // SAFETY: `statvfs` is a plain C struct of integers, so an all-zero bit pattern is a
        // valid (if meaningless) value; `statvfs(3)` overwrites every field it defines before we
        // read any of it. The pointer passed in comes from a `CString` that outlives the call and
        // is guaranteed NUL-terminated with no interior NUL, which is the function's only
        // precondition. A non-zero return means the struct was not populated, and we take the
        // error path instead of reading it.
        #[allow(unsafe_code)]
        let (rc, stat) = unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let rc = libc::statvfs(c_path.as_ptr(), &mut stat);
            (rc, stat)
        };

        if rc == 0 {
            // `fsblkcnt_t` and `f_frsize` are u64 on Linux but narrower on some targets
            // (32-bit musl, macOS). The casts are redundant here and load-bearing elsewhere.
            #[allow(clippy::unnecessary_cast)]
            let (total, available) = {
                let frsize = stat.f_frsize as u64;
                (
                    (stat.f_blocks as u64).saturating_mul(frsize),
                    (stat.f_bavail as u64).saturating_mul(frsize),
                )
            };
            return Ok((Some(total), Some(available)));
        }
        Err(std::io::Error::last_os_error().into())
    }
    #[cfg(not(target_family = "unix"))]
    {
        Ok((None, None))
    }
}

/// Available bytes on the filesystem backing `path`.
pub fn free_bytes(path: &str) -> Result<Option<u64>> {
    stats(path).map(|(_, available)| available)
}
