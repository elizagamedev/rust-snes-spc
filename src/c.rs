use libc;

pub type SnesSpc = libc::c_void;
pub type SpcSample = libc::c_short;
pub type SpcTime = libc::c_int;
pub type SpcErr = *const libc::c_char;
pub type SpcFilter = libc::c_void;
pub type SpcDsp = libc::c_void;
pub type CopyFunc = extern "C" fn(io: *mut libc::c_void,
                                  state: *const libc::c_void,
                                  size: libc::size_t);

extern "C" {
    // Creates new SPC emulator. NULL if out of memory.
    pub fn spc_new() -> *mut SnesSpc;
    // Frees SPC emulator
    pub fn spc_delete(obj: *mut SnesSpc);

    // **** Emulator use ****

    // Sets IPL ROM data. Library does not include ROM data. Most SPC music files
    // don't need ROM, but a full emulator must provide this.
    pub fn spc_init_rom(obj: *mut SnesSpc, rom: *const libc::c_uchar);

    // Sets destination for output samples
    pub fn spc_set_output(obj: *mut SnesSpc, out: *mut SpcSample, out_size: libc::c_int);

    // Number of samples written to output since last set
    pub fn spc_sample_count(obj: *const SnesSpc) -> libc::c_int;

    // Resets SPC to power-on state. This resets your output buffer, so you must
    // call spc_set_output() after this.
    pub fn spc_reset(obj: *mut SnesSpc);

    // Emulates pressing reset switch on SNES. This resets your output buffer, so
    // you must call spc_set_output() after this.
    pub fn spc_soft_reset(obj: *mut SnesSpc);

    // Reads/writes port at specified time
    pub fn spc_read_port(obj: *mut SnesSpc, time: SpcTime, port: libc::c_int) -> libc::c_int;
    pub fn spc_write_port(obj: *mut SnesSpc, time: SpcTime, port: libc::c_int, data: libc::c_int);

    // Runs SPC to end_time and starts a new time frame at 0
    pub fn spc_end_frame(obj: *mut SnesSpc, end_time: SpcTime);


    // **** Sound control ****

    // Mutes voices corresponding to non-zero bits in mask. Reduces emulation accuracy.
    pub fn spc_mute_voices(obj: *mut SnesSpc, mask: libc::c_int);

    // If true, prevents channels and global volumes from being phase-negated.
    // Only supported by fast DSP; has no effect on accurate DSP.
    pub fn spc_disable_surround(obj: *mut SnesSpc, disable: libc::c_int);

    // Sets tempo, where spc_tempo_unit = normal, spc_tempo_unit / 2 = half speed, etc.
    pub fn spc_set_tempo(obj: *mut SnesSpc, tempo: libc::c_int);


    // **** SPC music playback ****

    // Loads SPC data into emulator. Returns NULL on success, otherwise error string.
    pub fn spc_load_spc(obj: *mut SnesSpc,
                        spc_in: *const libc::c_void,
                        size: libc::c_long)
                        -> SpcErr;

    // Clears echo region. Useful after loading an SPC as many have garbage in echo.
    pub fn spc_clear_echo(obj: *mut SnesSpc);

    // Plays for count samples and write samples to out. Discards samples if out
    // is NULL. Count must be a multiple of 2 since output is stereo.
    pub fn spc_play(obj: *mut SnesSpc, count: libc::c_int, out: *mut libc::c_short) -> SpcErr;

    // Skips count samples. Several times faster than spc_play().
    pub fn spc_skip(obj: *mut SnesSpc, count: libc::c_int) -> SpcErr;


    // **** State save/load (only available with accurate DSP) ****

    // Saves/loads exact emulator state
    pub fn spc_copy_state(obj: *mut SnesSpc, io: *mut libc::c_void, copy_func: CopyFunc);

    // Writes minimal SPC file header to spc_out
    pub fn spc_init_header(spc_out: *mut libc::c_void);

    // Saves emulator state as SPC file data. Writes spc_file_size bytes to spc_out.
    // Does not set up SPC header; use spc_init_header() for that.
    pub fn spc_save_spc(obj: *mut SnesSpc, spc_out: *mut libc::c_void);

    // Returns non-zero if new key-on events occurred since last check. Useful for
    // trimming silence while saving an SPC.
    pub fn spc_check_kon(obj: *mut SnesSpc) -> libc::c_int;


    // **** SPC_Filter ****

    // Creates new filter. NULL if out of memory.
    pub fn spc_filter_new() -> *mut SpcFilter;

    // Frees filter
    pub fn spc_filter_delete(obj: *mut SpcFilter);

    // Filters count samples of stereo sound in place. Count must be a multiple of 2.
    pub fn spc_filter_run(obj: *mut SpcFilter, io: *mut SpcSample, count: libc::c_int);

    // Clears filter to silence
    pub fn spc_filter_clear(obj: *mut SpcFilter);

    // Sets gain (volume), where spc_filter_gain_unit is normal. Gains greater than
    // spc_filter_gain_unit are fine, since output is clamped to 16-bit sample range.
    pub fn spc_filter_set_gain(obj: *mut SpcFilter, gain: libc::c_int);

    // Sets amount of bass (logarithmic scale)
    pub fn spc_filter_set_bass(obj: *mut SpcFilter, bass: libc::c_int);

    // **** SPC_DSP ****

    // Creates new DSP emulator. NULL if out of memory.
    pub fn spc_dsp_new() -> *mut SpcDsp;

    // Frees DSP emulator
    pub fn spc_dsp_delete(obj: *mut SpcDsp);

    // Initializes DSP and has it use the 64K RAM provided
    pub fn spc_dsp_init(obj: *mut SpcDsp, ram_64k: *mut libc::c_void);

    // Sets destination for output samples. If out is NULL or out_size is 0,
    // doesn't generate any.
    pub fn spc_dsp_set_output(obj: *mut SpcDsp, out: *mut SpcSample, out_size: libc::c_int);

    // Number of samples written to output since it was last set, always
    // a multiple of 2. Undefined if more samples were generated than
    // output buffer could hold.
    pub fn spc_dsp_sample_count(obj: *const SpcDsp) -> libc::c_int;


    // **** Emulation ****

    // Resets DSP to power-on state
    pub fn spc_dsp_reset(obj: *mut SpcDsp);

    // Emulates pressing reset switch on SNES
    pub fn spc_dsp_soft_reset(obj: *mut SpcDsp);

    // Reads/writes DSP registers. For accuracy, you must first call spc_dsp_run()
    // to catch the DSP up to present.
    pub fn spc_dsp_read(obj: *const SpcDsp, addr: libc::c_int) -> libc::c_int;
    pub fn spc_dsp_write(obj: *const SpcDsp, addr: libc::c_int, data: libc::c_int);

    // Runs DSP for specified number of clocks (~1024000 per second). Every 32 clocks
    // a pair of samples is be generated.
    pub fn spc_dsp_run(obj: *mut SpcDsp, clock_count: libc::c_int);


    // **** Sound control ****

    // Mutes voices corresponding to non-zero bits in mask. Reduces emulation accuracy.
    pub fn spc_dsp_mute_voices(obj: *mut SpcDsp, mask: libc::c_int);

    // If true, prevents channels and global volumes from being phase-negated.
    // Only supported by fast DSP; has no effect on accurate DSP.
    pub fn spc_dsp_disable_surround(obj: *mut SpcDsp, disable: libc::c_int);


    // **** State save/load ****

    // Resets DSP and uses supplied values to initialize registers
    pub fn spc_dsp_load(obj: *mut SpcDsp, regs: *const libc::c_uchar);

    // Saves/loads exact emulator state (accurate DSP only)
    pub fn spc_dsp_copy_state(obj: *mut SpcDsp, io: *mut libc::c_void, copy_func: CopyFunc);

    // Returns non-zero if new key-on events occurred since last call (accurate DSP only)
    pub fn spc_dsp_check_kon(obj: *mut SpcDsp) -> libc::c_int;
}
