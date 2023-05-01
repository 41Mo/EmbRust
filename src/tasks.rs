use crate::{serial::TELEM1, led::LEDS};
use freertos_rust::{Duration, CurrentTask};

pub fn default_task() {
    loop {
        cortex_m::asm::nop();
    }
}

pub fn telem1rw() {
    loop {
        let byte = TELEM1.read();
        TELEM1.write(byte);
    }
}

pub fn blink() {
    loop {
        LEDS.toggle_green();
        CurrentTask::delay(Duration::ms(1000));
    }
}
