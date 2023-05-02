use boards::periph::{write_serial, HAL};
use freertos_rust::{CurrentTask, Duration};

use boards::hal::{
    prelude::*,
    stm32,
    usb_hs::{UsbBus, USB2},
};
use usb_device::prelude::*;

extern crate alloc;
use alloc::boxed::Box;
use core::{ptr::{self, null_mut}, mem::size_of};

pub fn default_task() {
    loop {}
}

pub fn telem1rw() {
    loop {
        // let byte = TELEM1.read();
        // TELEM1.write(byte);
    }
}

pub fn blink() {
    let led_green = unsafe { HAL.take_led_green().unwrap() };
    loop {
        // LEDS.toggle_green();
        led_green.toggle();
        CurrentTask::delay(Duration::ms(1000));
    }
}

pub fn get_usb() {
}

pub fn usb_read()
{
}
