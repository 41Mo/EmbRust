use stm32h7xx_hal::{
    gpio::*,
    pac,
    prelude::*,
    stm32::*,
    serial::{Tx, Rx},
};
use crate::led::Leds;
use crate::serial::*;
use crate::uart_ops;
use core::sync::atomic::{AtomicPtr, Ordering};
use nb::block;

pub type LedBlueType = Pin<'E', 3, Output<PushPull>>;
pub type LedGreenType = Pin<'E', 4, Output<PushPull>>;

pub static TELEM1: UartType<UART7> = UartType {
    tx: AtomicPtr::new(core::ptr::null_mut()),
    rx: AtomicPtr::new(core::ptr::null_mut()),
};

pub static LEDS: Leds<LedBlueType, LedGreenType> = Leds {
    led_blue: AtomicPtr::new(core::ptr::null_mut()),
    led_green: AtomicPtr::new(core::ptr::null_mut()),
};

uart_ops! {
    UART7
}

pub fn setup_periph() {
    let dp = pac::Peripherals::take().unwrap();
    let pwrcfg = dp.PWR.constrain().freeze();
    let rcc = dp.RCC.constrain();

    let ccdr = rcc
        .sys_ck(200.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    LEDS.init(
        gpioe.pe4.into_push_pull_output(),
        gpioe.pe3.into_push_pull_output(),
    );

    let rx = gpioe.pe7.into_alternate::<7>();
    let tx = gpioe.pe8.into_alternate::<7>();
    let serial7 = dp
        .UART7
        .serial((tx, rx), 57_600.bps(), ccdr.peripheral.UART7, &ccdr.clocks)
        .unwrap();

    TELEM1.from_serial(serial7.split());
}
