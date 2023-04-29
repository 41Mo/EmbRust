#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use cortex_m::asm;
use cortex_m_rt::exception;
use cortex_m_rt::{entry, ExceptionFrame};
use embedded_hal::digital::v2::OutputPin;
use freertos_rust::*;
use core::alloc::Layout;
use stm32h7xx_hal::gpio::*;

use cortex_m;
use stm32h7xx_hal as hal;

pub use crate::hal::{pac, prelude::*};

extern crate panic_halt; // panic handler

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

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

pub struct GpioPin<D1: OutputPin> {
    d1: D1,
}

impl<D1: OutputPin> GpioPin<D1>
{
    pub fn from_pins(d1: D1) -> GpioPin<D1> {
        GpioPin {
            d1
        }
    }
    pub fn set_led(&mut self,on:bool){
        if on {
            self.d1.set_high();
        } else {
            self.d1.set_low();
        }
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let mut device = GpioPin::from_pins(gpioe.pe3.into_push_pull_output());
    device.set_led(false);
    Task::new().name("Blinky").stack_size(128).priority(TaskPriority(2)).start(move || {
        loop{
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            device.set_led(true);
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            device.set_led(false);
        }
    }).unwrap();
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
fn vApplicationStackOverflowHook(pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
