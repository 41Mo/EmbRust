use std::env;
use std::path::PathBuf;
use std::fs::copy;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    copy("memory.x", PathBuf::from(out_dir.as_str()).join("memory.x"),).unwrap();

    let mut b = freertos_cargo_build::Builder::new();

    // Path to FreeRTOS kernel or set ENV "FREERTOS_SRC" instead
    b.freertos("FreeRTOS-Kernel");
    b.freertos_config("src");       // Location of `FreeRTOSConfig.h` 
    b.freertos_port("GCC/ARM_CM7/r0p1".to_string()); // Port dir relativ to 'FreeRTOS-Kernel/portable' 
    // b.heap("heap_4.c");             // Set the heap_?.c allocator to use from 
                                    // 'FreeRTOS-Kernel/portable/MemMang' (Default: heap_4.c)       

    // b.get_cc().file("More.c");   // Optional additional C-Code to be compiled

    b.compile().unwrap_or_else(|e| { panic!("{}", e.to_string()) });
}
