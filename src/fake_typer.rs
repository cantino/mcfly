use libc;
use std::convert::TryInto;

// Should we be using https://docs.rs/libc/0.2.44/libc/fn.ioctl.html instead?
extern "C" {
    pub fn ioctl(fd: libc::c_int, request: libc::c_ulong, arg: ...) -> libc::c_int;
}

#[allow(clippy::useless_conversion)]
pub fn use_tiocsti(string: &str) {
    for byte in string.as_bytes() {
        let a: *const u8 = byte;
        if unsafe { ioctl(0, libc::TIOCSTI.try_into().unwrap(), a) } < 0 {
            panic!("Error encountered when calling ioctl");
        }
    }
}
