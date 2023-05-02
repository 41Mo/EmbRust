#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

mod tasks;

use boards::periph::{HAL, HALDATA};
use core::{
    alloc::Layout,
    sync::atomic::{self, AtomicPtr, Ordering},
};
use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use freertos_rust::*;
use tasks::{blink, default_task, telem1rw, usb_read};
extern crate panic_halt; // panic handler

use boards::hal::{
    prelude::*,
    stm32,
    usb_hs::{UsbBus, USB2},
};
use usb_device::prelude::*;

extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::{
    mem::size_of,
    ptr::{self, null_mut},
};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    let mut hd: HALDATA = HALDATA::setup();

    unsafe {
        HAL.freeze(&mut hd);
    }

    let mut usb_bus = UsbBus::new(hd.usb, unsafe { &mut EP_MEMORY });
    let ap = AtomicPtr::new(null_mut());
    ap.store(&mut usb_bus, Ordering::Relaxed);
    let ausb_bus = Arc::new(ap);
    let t1 = unsafe { ausb_bus.as_ref().load(Ordering::Relaxed).as_ref().unwrap() };

    let mut serial = usbd_serial::SerialPort::new(t1);
    let mut usb_dev = UsbDeviceBuilder::new(t1, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST PORT 2")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    let ap = AtomicPtr::new(null_mut());
    ap.store(&mut serial, Ordering::Relaxed);
    let shared_usb_serial = Arc::new(ap);
    let ap = AtomicPtr::new(null_mut());
    ap.store(&mut usb_dev, Ordering::Relaxed);
    let shared_usb_dev = Arc::new(ap);


    Task::new()
        .name("SerialWrite")
        .stack_size(256)
        .priority(TaskPriority(1))
        .start(telem1rw)
        .unwrap();

    Task::new()
        .name("Blinky")
        .stack_size(128)
        .priority(TaskPriority(1))
        .start(blink)
        .unwrap();
    Task::new()
        .name("Telem0")
        .stack_size(1024)
        .priority(TaskPriority(1))
        .start(move || {
            let usb_dev = unsafe {
                shared_usb_dev
                    .as_ref()
                    .load(Ordering::Relaxed)
                    .as_mut()
                    .unwrap()
            };
            let serial = unsafe {
                shared_usb_serial
                    .as_ref()
                    .load(Ordering::Relaxed)
                    .as_mut()
                    .unwrap()
            };
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
        })
        .unwrap();

    FreeRtosUtils::start_scheduler();
}

fn delay_n(n: i32) {
    for _ in 0..n {
        {
            let mut _i = 0;
            for _ in 0..2_00 {
                _i += 1;
            }
        }
    }
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    // custom default handler
    // irqn is negative for Cortex-M exceptions
    // irqn is positive for device specific (line IRQ)
    let lb = HAL.take_led_blue().unwrap();
    let lg = HAL.take_led_green().unwrap();
    lb.set_low();
    lg.set_low();
    panic!("Exception: {}", _irqn);
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    // Blink 3 times long when exception occures
    let lb = unsafe { HAL.take_led_blue().unwrap() };
    lb.set_high();
    delay_n(1000);
    for _ in 0..3 {
        lb.toggle();
        delay_n(5000);
    }
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    let lb = unsafe { HAL.take_led_blue().unwrap() };
    let lg = unsafe { HAL.take_led_green().unwrap() };
    lb.set_high();
    lb.set_low();

    loop {
        lb.toggle();
        lg.toggle();
    }
    asm::bkpt();
}

#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {
    let lb = unsafe { HAL.take_led_blue().unwrap() };
    let lg = unsafe { HAL.take_led_green().unwrap() };
    for _ in 0..10 {
        lb.toggle();
        delay_n(1000);
    }
    lb.set_low();
    asm::bkpt();
}
