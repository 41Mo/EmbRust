#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use core::alloc::Layout;
use core::fmt::Write;
use cortex_m::asm;
use cortex_m_rt::exception;
use cortex_m_rt::{entry, ExceptionFrame};
use freertos_rust::*;

use nb::block;

use cortex_m;
use stm32h7xx_hal as hal;

use crate::hal::{
    gpio::*,
    pac,
    prelude::*,
    serial::{Rx, Serial, SerialExt, Tx},
    stm32::*,
    time::*,
};

use core::sync::atomic::{AtomicPtr, Ordering};

extern crate panic_halt; // panic handler

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

struct UartType<T> {
    tx: AtomicPtr<Tx<T>>,
    rx: AtomicPtr<Rx<T>>,
}

static SERIAL7: UartType<UART7> = UartType {
    tx: AtomicPtr::new(core::ptr::null_mut()),
    rx: AtomicPtr::new(core::ptr::null_mut()),
};

impl<T> UartType<T> {}

macro_rules! uart_ops {
    ($usartX:ident) => {
        impl UartType<$usartX> {
            pub fn from_serial(&self, mut serial: (Tx<$usartX>, Rx<$usartX>)) {
                self.rx.store(&mut serial.1, Ordering::Relaxed);
                self.tx.store(&mut serial.0, Ordering::Relaxed);
            }
            pub fn write(&self, byte: u8) {
                unsafe {
                    block!(self.tx.load(Ordering::Relaxed).as_mut().unwrap().write(byte)).unwrap()
                }
            }
            fn read(&self) -> u8 {
                unsafe {
                    block!(self.rx.load(Ordering::Relaxed).as_mut().unwrap().read()).unwrap()
                }
            }
        }
    };
}

uart_ops! {
    UART7
}

fn delay() {
    let mut _i = 0;
    for _ in 0..2_00 {
        _i += 1;
    }
}

fn delay_n(n: i32) {
    for _ in 0..n {
        delay();
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let pwrcfg = dp.PWR.constrain().freeze();
    let rcc = dp.RCC.constrain();

    let ccdr = rcc
        .sys_ck(200.MHz())
        .pll1_q_ck(200.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    let mut led_blue = gpioe.pe3.into_push_pull_output().internal_pull_down(true);
    led_blue.set_high();
    let mut led_green = gpioe.pe4.into_push_pull_output();
    led_green.set_low();

    let rx = gpioe.pe7.into_alternate::<7>();
    let tx = gpioe.pe8.into_alternate::<7>();
    let serial7 = dp
        .UART7
        .serial((tx, rx), 57_600.bps(), ccdr.peripheral.UART7, &ccdr.clocks)
        .unwrap();

    SERIAL7.from_serial(serial7.split());

    Task::new()
        .name("Defaul")
        .stack_size(128)
        .priority(TaskPriority(1))
        .start(move || loop {
            cortex_m::asm::nop();
        })
        .unwrap();

    Task::new()
        .name("SerialWrite")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(move || loop {
            SERIAL7.write(SERIAL7.read());
        })
        .unwrap();

    Task::new()
        .name("Blinky")
        .stack_size(128)
        .priority(TaskPriority(3))
        .start(move || loop {
            if led_green.is_set_high() {
                led_green.set_low();
            } else {
                led_green.set_high();
            }
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
        })
        .unwrap();

    FreeRtosUtils::start_scheduler();
}

#[exception]
fn DefaultHandler(_irqn: i16) {
    // custom default handler
    // irqn is negative for Cortex-M exceptions
    // irqn is positive for device specific (line IRQ)
    // set_led(true);(true);
    // panic!("Exception: {}", irqn);
}

#[exception]
fn HardFault(_ef: &ExceptionFrame) -> ! {
    // Blink 3 times long when exception occures
    delay_n(10);
    for _ in 0..3 {
        // set_led(true);
        // delay_n(1000);
        // set_led(false);
        // delay_n(555);
    }
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    //set_led(true);
    asm::bkpt();
    loop {}
}

#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
