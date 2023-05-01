#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

mod tasks;

use boards::periph::{HAL, HALDATA};
use core::alloc::Layout;
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

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

#[entry]
fn main() -> ! {
    let mut hd: HALDATA = HALDATA::setup();
    unsafe {
        HAL.freeze(&mut hd);
    }

    //
    // let mut buf = [0u8; 64];

    // Task::new()
    //     .name("Default")
    //     .stack_size(1)
    //     .priority(TaskPriority(1))
    //     .start(default_task)
    //     .unwrap();
    //
    // Task::new()
    //     .name("SerialWrite")
    //     .stack_size(256)
    //     .priority(TaskPriority(2))
    //     .start(telem1rw)
    //     .unwrap();

    Task::new()
        .name("Blinky")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(blink)
        .unwrap();

    Task::new()
        .name("Telem0")
        .stack_size(1024)
        .priority(TaskPriority(1))
        .start(usb_read)
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
