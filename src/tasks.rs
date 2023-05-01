use crate::periph::{LEDS, TELEM1};
use freertos_rust::{Duration, CurrentTask};

pub fn default_task() {
    loop {
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
