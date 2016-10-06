extern crate libc;
mod c;

use std::ffi::CStr;
use std::ptr;
use std::mem;
use std::error;
use std::fmt;
use std::slice;

pub struct Spc {
    cobj: *mut c::SnesSpc,
    // pub out: Vec<Sample>,
    buffer_a: Box<[Sample]>,
    buffer_b: Box<[Sample]>,
    buffer_a_active: bool,
    buffer_enabled: bool,
}

impl Spc {
    pub fn new(buffer_size: usize) -> Spc {
        assert!(buffer_size % 2 == 0, "buffer_size must be even");
        unsafe {
            let cobj: *mut c::SnesSpc = c::spc_new();
            if cobj.is_null() {
                panic!("Out of memory");
            }
            Spc {
                cobj: cobj,
                buffer_a: vec![0; buffer_size].into_boxed_slice(),
                buffer_b: vec![0; buffer_size].into_boxed_slice(),
                buffer_a_active: true,
                buffer_enabled: false,
            }
        }
    }

    pub fn init_rom(&mut self, rom: &[u8]) {
        assert!(rom.len() == ROM_SIZE);
        unsafe {
            c::spc_init_rom(self.cobj, rom.as_ptr());
        }
    }

    pub fn enable_buffer(&mut self, val: bool) {
        if self.buffer_enabled == val {
            return;
        }
        self.buffer_a_active = true;
        unsafe {
            let (cout, csize) = if val {
                let cout = self.buffer_a.as_mut_ptr() as *mut Sample;
                let csize = self.buffer_a.len() as libc::c_int;
                (cout, csize)
            } else {
                let cout = mem::transmute::<*const Sample, *mut Sample>(ptr::null());
                (cout, 0)
            };
            c::spc_set_output(self.cobj, cout, csize);
        }
        self.buffer_enabled = val;
    }

    pub fn flush_buffer(&mut self) -> &[Sample] {
        assert!(self.buffer_enabled);
        let sample_count = self.sample_count();

        let (result, active_buffer) = if self.buffer_a_active {
            self.buffer_a_active = false;
            (&self.buffer_a[0..sample_count], &mut self.buffer_b)
        } else {
            self.buffer_a_active = true;
            (&self.buffer_b[0..sample_count], &mut self.buffer_a)
        };

        unsafe {
            let cout = active_buffer.as_mut_ptr() as *mut Sample;
            let csize = active_buffer.len() as libc::c_int;
            c::spc_set_output(self.cobj, cout, csize);
        }
        result
    }

    pub fn sample_count(&self) -> usize {
        unsafe { c::spc_sample_count(self.cobj) as usize }
    }

    pub fn read_port(&mut self, time: usize, port: u32) -> u8 {
        unsafe { c::spc_read_port(self.cobj, time as c::SpcTime, port as libc::c_int) as u8 }
    }

    pub fn write_port(&mut self, time: usize, port: u32, data: u8) {
        unsafe {
            c::spc_write_port(self.cobj,
                              time as c::SpcTime,
                              port as libc::c_int,
                              data as libc::c_int);
        }
    }

    pub fn end_frame(&mut self, end_time: usize) {
        unsafe {
            c::spc_end_frame(self.cobj, end_time as c::SpcTime);
        }
    }

    pub fn reset(&mut self) {
        unsafe { c::spc_reset(self.cobj) }
    }

    pub fn soft_reset(&mut self) {
        unsafe { c::spc_soft_reset(self.cobj) }
    }

    pub fn mute_voices(&mut self, mask: u32) {
        unsafe {
            c::spc_mute_voices(self.cobj, mask as libc::c_int);
        }
    }

    pub fn disable_surround(&mut self, disable: bool) {
        unsafe {
            let cbool: libc::c_int = if disable { 1 } else { 0 };
            c::spc_disable_surround(self.cobj, cbool);
        }
    }

    pub fn set_tempo(&mut self, tempo: u32) {
        unsafe {
            c::spc_set_tempo(self.cobj, tempo as libc::c_int);
        }
    }

    pub fn check_kon(&mut self) -> bool {
        unsafe { c::spc_check_kon(self.cobj) != 0 }
    }

    pub fn copy_state(&mut self) -> Box<[u8]> {
        extern "C" fn callback(io: *mut libc::c_void,
                               state: *const libc::c_void,
                               size: libc::size_t) {
            unsafe {
                let vec = mem::transmute::<*mut libc::c_void, &mut Vec<u8>>(io);
                vec.extend(slice::from_raw_parts(state as *const u8, size as usize));
            }
        }

        unsafe {
            let mut buf: Vec<u8> = Vec::with_capacity(SPC_STATE_SIZE);
            let ptr = mem::transmute::<&mut Vec<u8>, *mut libc::c_void>(&mut buf);
            c::spc_copy_state(self.cobj, ptr, callback);
            buf.into_boxed_slice()
        }
    }

    pub fn save_spc(&mut self) -> [u8; SPC_FILE_SIZE] {
        unsafe {
            let mut buf: [u8; SPC_FILE_SIZE];
            buf = mem::uninitialized();
            let ptr = buf.as_mut_ptr() as *mut libc::c_void;
            c::spc_init_header(ptr);
            c::spc_save_spc(self.cobj, ptr);
            buf
        }
    }
}

impl Drop for Spc {
    fn drop(&mut self) {
        unsafe {
            c::spc_delete(self.cobj);
        }
    }
}

pub struct SpcPlayer {
    cobj: *mut c::SnesSpc,
}

impl SpcPlayer {
    pub fn new() -> SpcPlayer {
        unsafe {
            let cobj: *mut c::SnesSpc = c::spc_new();
            if cobj.is_null() {
                panic!("Out of memory");
            }
            SpcPlayer { cobj: cobj }
        }
    }

    pub fn load_spc(&mut self, spc_in: &[u8]) -> SpcResult {
        unsafe {
            let cin = spc_in.as_ptr() as *const libc::c_void;
            let csize = spc_in.len() as libc::c_long;
            wrap_error(c::spc_load_spc(self.cobj, cin, csize))
        }
    }

    pub fn clear_echo(&mut self) {
        unsafe {
            c::spc_clear_echo(self.cobj);
        }
    }

    pub fn play(&mut self, out: &mut [i16]) -> SpcResult {
        unsafe {
            let cout = out.as_mut_ptr() as *mut libc::c_short;
            let csize = out.len() as libc::c_int;
            wrap_error(c::spc_play(self.cobj, csize, cout))
        }
    }

    pub fn skip(&mut self, count: usize) -> SpcResult {
        unsafe { wrap_error(c::spc_skip(self.cobj, count as libc::c_int)) }
    }

    pub fn mute_voices(&mut self, mask: u32) {
        unsafe {
            c::spc_mute_voices(self.cobj, mask as libc::c_int);
        }
    }

    pub fn disable_surround(&mut self, disable: bool) {
        unsafe {
            let cbool: libc::c_int = if disable { 1 } else { 0 };
            c::spc_disable_surround(self.cobj, cbool);
        }
    }

    pub fn set_tempo(&mut self, tempo: u32) {
        unsafe {
            c::spc_set_tempo(self.cobj, tempo as libc::c_int);
        }
    }

    pub fn check_kon(&mut self) -> bool {
        unsafe { c::spc_check_kon(self.cobj) != 0 }
    }
}

impl Drop for SpcPlayer {
    fn drop(&mut self) {
        unsafe {
            c::spc_delete(self.cobj);
        }
    }
}

pub struct Filter {
    cobj: *mut c::SpcFilter,
}

impl Filter {
    pub fn new() -> Filter {
        unsafe {
            let cobj: *mut c::SpcFilter = c::spc_filter_new();
            if cobj.is_null() {
                panic!("Out of memory");
            }
            Filter { cobj: cobj }
        }
    }

    pub fn run(&mut self, io: &mut [i16]) {
        unsafe {
            let cio = io.as_mut_ptr() as *mut libc::c_short;
            let csize = io.len() as libc::c_int;
            c::spc_filter_run(self.cobj, cio, csize);
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            c::spc_filter_clear(self.cobj);
        }
    }

    pub fn set_gain(&mut self, gain: u32) {
        unsafe {
            c::spc_filter_set_gain(self.cobj, gain as libc::c_int);
        }
    }

    pub fn set_bass(&mut self, bass: u32) {
        unsafe {
            c::spc_filter_set_bass(self.cobj, bass as libc::c_int);
        }
    }
}

impl Drop for Filter {
    fn drop(&mut self) {
        unsafe {
            c::spc_filter_delete(self.cobj);
        }
    }
}

type SpcResult = Result<(), Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Error {
        Error { message: message }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&error::Error> {
        Option::None
    }
}

unsafe fn wrap_error(string: *const libc::c_char) -> Result<(), Error> {
    if string.is_null() {
        Ok(())
    } else {
        let string = CStr::from_ptr(string);
        let string = string.to_str().unwrap().to_string();
        Err(Error::new(string))
    }
}

// Types
pub type Sample = i16;

// Constants
pub const SAMPLE_RATE: u32 = 32000;
pub const ROM_SIZE: usize = 0x40;
pub const CLOCK_RATE: u32 = 1024000;
pub const CLOCKS_PER_SAMPLE: usize = 32;
pub const PORT_COUNT: u32 = 4;
pub const VOICE_COUNT: u32 = 8;
pub const TEMPO_UNIT: u32 = 0x100;
pub const SPC_STATE_SIZE: usize = 67 * 1024;
pub const SPC_FILE_SIZE: usize = 0x10200;
pub const FILTER_GAIN_UNIT: u32 = 0x100;
pub const FILTER_BASS_NONE: u32 = 0;
pub const FILTER_BASS_NORM: u32 = 8;
pub const FILTER_BASS_MAX: u32 = 31;
pub const DSP_REGISTER_COUNT: u32 = 128;
pub const DSP_STATE_SIZE: usize = 640;
pub const ROM: [u8; ROM_SIZE] = [0xcd, 0xef, 0xbd, 0xe8, 0x00, 0xc6, 0x1d, 0xd0, 0xfc, 0x8f, 0xaa,
                                 0xf4, 0x8f, 0xbb, 0xf5, 0x78, 0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19,
                                 0xeb, 0xf4, 0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0x0b, 0xe4, 0xf5, 0xcb,
                                 0xf4, 0xd7, 0x00, 0xfc, 0xd0, 0xf3, 0xab, 0x01, 0x10, 0xef, 0x7e,
                                 0xf4, 0x10, 0xeb, 0xba, 0xf6, 0xda, 0x00, 0xba, 0xf4, 0xc4, 0xf4,
                                 0xdd, 0x5d, 0xd0, 0xdb, 0x1f, 0x00, 0x00, 0xc0, 0xff];
