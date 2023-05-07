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
use tasks::{blink, console};

extern crate alloc;
extern crate panic_halt; // panic handler

use alloc::alloc::GlobalAlloc;
use alloc::*;
use core::ptr::null_mut;

pub struct CustomAllocator {}

static mut ALLOCATED_MEM: usize = 0;

unsafe impl GlobalAlloc for CustomAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // ALLOCATED_MEM += layout.size();
        let res = freertos_rs_pvPortMalloc(layout.size() as u32);
        return res as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // ALLOCATED_MEM -= _layout.size();
        freertos_rs_vPortFree(ptr as FreeRtosVoidPtr)
    }
}

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator {};

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
    t1: AtomicPtr<Task>,
    t2: AtomicPtr<Task>,
}

lazy_static::lazy_static! {
    pub static ref TASK_HANDLES:TaskHandles = TaskHandles{t1:AtomicPtr::new(null_mut()), t2:AtomicPtr::new(null_mut())};
}

#[entry]
fn main() -> ! {
    lazy_static::initialize(&HAL);
    // asm::bkpt();

    Task::new()
        .name("Blinky")
        .stack_size(256)
        .priority(TaskPriority(1))
        .start(blink)
        .unwrap();

    Task::new()
        .name("Telem0")
        .stack_size(10*1024/4)
        .priority(TaskPriority(1))
        .start(console)
        .unwrap();

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
fn vApplicationStackOverflowHook(pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    let lb = unsafe { HAL.led_blue.load(Ordering::Relaxed).as_mut().unwrap() };
    let _lg = unsafe { HAL.led_green.load(Ordering::Relaxed).as_mut().unwrap() };
    for _ in 0..10 {
        lb.toggle();
        delay_n(1000);
    }
    lb.set_low();
    asm::bkpt();
}
