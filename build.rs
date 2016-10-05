extern crate gcc;

fn main() {
    gcc::Config::new()
        .cpp(true)
        .include("snes_spc/snes_spc")
        .flag("-fno-rtti")
        .flag("-fno-exceptions")
        .file("snes_spc/snes_spc/dsp.cpp")
        .file("snes_spc/snes_spc/SNES_SPC_misc.cpp")
        .file("snes_spc/snes_spc/SNES_SPC_state.cpp")
        .file("snes_spc/snes_spc/SNES_SPC.cpp")
        .file("snes_spc/snes_spc/SPC_DSP.cpp")
        .file("snes_spc/snes_spc/SPC_Filter.cpp")
        .file("snes_spc/snes_spc/spc.cpp")
        .flag("-O3")
        .compile("libsnes_spc.a");
}
