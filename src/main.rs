#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

mod tasks;

use boards::periph::HAL;
use core::{
    alloc::Layout,
    sync::atomic::{AtomicPtr, Ordering},
};
use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use freertos_rust::*;
use tasks::{blink, usb_read};

extern crate panic_halt; // panic handler
extern crate alloc;


use core::ptr::null_mut;

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;


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

pub struct TaskHandles {
    t1:AtomicPtr<Task>,
    t2:AtomicPtr<Task>,
}

lazy_static::lazy_static! {
    pub static ref TASK_HANDLES:TaskHandles = TaskHandles{t1:AtomicPtr::new(null_mut()), t2:AtomicPtr::new(null_mut())};
}

#[entry]
fn main() -> ! {
    lazy_static::initialize(&HAL);

    let mut t2 = Task::new()
        .name("Blinky")
        .stack_size(256)
        .start(blink)
        .unwrap();
    let mut t1 = Task::new()
        .name("Telem0")
        .stack_size(1024)
        .start(usb_read)
        .unwrap();

    TASK_HANDLES.t1.store(&mut t1, Ordering::Relaxed);
    TASK_HANDLES.t2.store(&mut t2, Ordering::Relaxed);

    FreeRtosUtils::start_scheduler();
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    // custom default handler
    // irqn is negative for Cortex-M exceptions
    // irqn is positive for device specific (line IRQ)
    // let lb = HAL.take_led_blue().unwrap();
    // let lg = HAL.take_led_green().unwrap();
    // lb.set_low();
    // lg.set_low();
    panic!("Exception: {}", _irqn);
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    // Blink 3 times long when exception occures
    // let lb = unsafe { HAL.take_led_blue().unwrap() };
    // lb.set_high();
    // delay_n(1000);
    // for _ in 0..3 {
    //     lb.toggle();
    //     delay_n(5000);
    // }
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    // let lb = unsafe { HAL.take_led_blue().unwrap() };
    // let lg = unsafe { HAL.take_led_green().unwrap() };
    // lb.set_high();
    // lb.set_low();

    asm::bkpt();
    loop {
    //     lb.toggle();
    //     lg.toggle();
    }
}

#[no_mangle]
fn vApplicationStackOverflowHook(_px_task: FreeRtosTaskHandle, _pc_task_name: FreeRtosCharPtr) {
    let lb = unsafe { HAL.led_blue.load(Ordering::Relaxed).as_mut().unwrap() };
    let _lg = unsafe { HAL.led_green.load(Ordering::Relaxed).as_mut().unwrap() };
    for _ in 0..10 {
        lb.toggle();
        delay_n(1000);
    }
    lb.set_low();
    asm::bkpt();
}
