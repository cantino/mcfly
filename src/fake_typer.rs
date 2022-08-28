// Should we be using https://docs.rs/libc/0.2.44/libc/fn.ioctl.html instead?
extern "C" {
    pub fn ioctl(fd: i8, request: u32, arg: *const u8) -> i8;
}

#[cfg(not(windows))]
pub fn use_tiocsti(string: &str) {
    for byte in string.as_bytes() {
        let a: *const u8 = byte;
        if unsafe { ioctl(0, libc::TIOCSTI as u32, a) } < 0 {
            panic!("Error encountered when calling ioctl");
        }
    }
}

#[cfg(windows)]
pub fn use_tiocsti(string: &str) {
    autopilot::key::type_string(string, &[], 0.0, 0.0);
}

pub fn delete_chars(n : usize)
{
    for _ in 0..n {
        autopilot::key::tap(&autopilot::key::Code(autopilot::key::KeyCode::Backspace), &[], 0, 0);
    }
}
