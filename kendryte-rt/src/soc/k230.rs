#[cfg(all(feature = "k230", target_arch = "riscv32"))]
#[naked]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {}
