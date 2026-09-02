//! Filesystem capacity probing, the one `unsafe` site in `piramid-server`.

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

        // SAFETY: statvfs is a struct of integers, so all-zero is a valid bit pattern, and
        // statvfs(3) overwrites every field before we read it. The pointer comes from a CString
        // that outlives the call. A non-zero return takes the error path without reading.
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
