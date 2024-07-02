#[cfg(not(windows))]
use libc;

// Should we be using https://docs.rs/libc/0.2.44/libc/fn.ioctl.html instead?
#[cfg(not(windows))]
extern "C" {
    pub fn ioctl(fd: libc::c_int, request: libc::c_ulong, arg: ...) -> libc::c_int;
}

#[cfg(not(windows))]
#[allow(clippy::useless_conversion)]
pub fn use_tiocsti(string: &str) {
    for byte in string.as_bytes() {
        let a: *const u8 = byte;
        assert!(
            unsafe { ioctl(0, libc::TIOCSTI.try_into().unwrap(), a) } >= 0,
            "Error encountered when calling ioctl"
        );
    }
}

#[cfg(windows)]
pub fn use_tiocsti(string: &str) {
    autopilot::key::type_string(string, &[], 0.0, 0.0);
}
