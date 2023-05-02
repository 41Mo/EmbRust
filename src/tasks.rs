use boards::periph::HAL;
use freertos_rust::{CurrentTask, Duration, DurationTicks};

extern crate alloc;
use crate::TASK_HANDLES;
use alloc::string::ToString;

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

pub fn usb_read() {
    let usb = HAL
        .usb
        .as_ref()
        .unwrap()
        .load(core::sync::atomic::Ordering::Relaxed);
    let serial = unsafe { &mut (*usb).serial };
    let usb_dev = unsafe { &mut (*usb).device };
    let mut last_wake_time = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };

    loop {
        if usb_dev.poll(&mut [serial]) {
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

        let now = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };

        if now - last_wake_time > Duration::ms(1000).to_ticks() {
            let t1 = unsafe {
                TASK_HANDLES
                    .t1
                    .load(core::sync::atomic::Ordering::Relaxed)
                    .as_ref()
                    .unwrap()
            };

            let name = t1.get_name().unwrap();
            let stack_usage = t1.get_stack_high_water_mark();

            let buf = [
                "Task:",
                &name,
                "stack left:",
                &stack_usage.to_string(),
                "\n",
            ]
            .join(" ");
            let buf = buf.as_bytes();
            let mut write_offset = 0;
            let count = buf.len();
            while write_offset < count {
                match serial.write(&buf[write_offset..count]) {
                    Ok(len) if len > 0 => {
                        write_offset += len;
                    }
                    _ => {}
                }
            }
            last_wake_time = now;
        }
    }
}
