#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

mod led;

//#[cfg(board = "MatekH743")]
mod periph;
mod tasks;
mod serial;

use stm32h7xx_hal as hal;
use crate::periph::{setup_periph, LEDS};
use core::alloc::Layout;
use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use freertos_rust::*;
use tasks::{blink, default_task, telem1rw};
extern crate panic_halt; // panic handler

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

#[entry]
fn main() -> ! {
    setup_periph();

    Task::new()
        .name("Default")
        .stack_size(1)
        .priority(TaskPriority(1))
        .start(default_task)
        .unwrap();

    Task::new()
        .name("SerialWrite")
        .stack_size(256)
        .priority(TaskPriority(2))
        .start(telem1rw)
        .unwrap();

    Task::new()
        .name("Blinky")
        .stack_size(128)
        .priority(TaskPriority(3))
        .start(blink)
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
    LEDS.off();
    LEDS.toggle_blue();
    LEDS.toggle_green();
    panic!("Exception: {}", _irqn);
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    // Blink 3 times long when exception occures
    LEDS.off();
    delay_n(10000);
    for _ in 0..3 {
        LEDS.toggle_blue();
        delay_n(5000);
    }
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();
    LEDS.flasher();
    loop {
        LEDS.toggle_blue();
        LEDS.toggle_green();
    }
}

#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {
    LEDS.off();
    for _ in 0..10 {
        LEDS.toggle_blue();
        delay_n(1000);
    }
    LEDS.set_blue_on();
    asm::bkpt();

}
