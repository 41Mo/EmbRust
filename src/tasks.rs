use boards::periph::HAL;
use freertos_rust::{CurrentTask, Duration};

use boards::hal::{
    prelude::*,
    stm32,
    usb_hs::{UsbBus, USB2},
};
use cortex_m::asm;
use usb_device::prelude::*;

extern crate alloc;
use alloc::boxed::Box;
use core::{
    mem::size_of,
    ptr::{self, null_mut},
};

pub fn default_task() {
    loop {}
}

pub fn telem1rw() {
    loop {
    }
}

pub fn blink() {
    let led = unsafe {
        HAL.led_green
            .load(core::sync::atomic::Ordering::Relaxed)
            .as_mut()
            .unwrap()
    };
    loop {
        led.toggle();
        CurrentTask::delay(Duration::ms(1000));
    }
}

pub fn get_usb() {}

pub fn usb_read() {
    let usb = unsafe {
        HAL.usb.as_ref().unwrap()
            .load(core::sync::atomic::Ordering::Relaxed)
    };
    let serial = unsafe { &mut (*usb).serial};
    let usb_dev = unsafe { &mut (*usb).device };
    loop {
        if !usb_dev.poll(&mut [serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
