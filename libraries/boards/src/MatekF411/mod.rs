use crate::led::Leds;
use crate::serial::*;
use crate::uart_ops;
use core::sync::atomic::{AtomicPtr, Ordering};
use hal::{
    gpio::*,
    pac,
    pac::*,
    prelude::*,
    serial::Serial,
    serial::{Rx, Tx},
};
use nb::block;
use stm32f4xx_hal as hal;

pub type LedBlueType = Pin<'C', 13, Output<PushPull>>;
pub type LedGreenType = Pin<'C', 14, Output<PushPull>>;

pub static TELEM1: UartType<USART1> = UartType {
    tx: AtomicPtr::new(core::ptr::null_mut()),
    rx: AtomicPtr::new(core::ptr::null_mut()),
};

pub static LEDS: Leds<LedBlueType, LedGreenType> = Leds {
    led_blue: AtomicPtr::new(core::ptr::null_mut()),
    led_green: AtomicPtr::new(core::ptr::null_mut()),
};

uart_ops! {
    USART1
}

pub fn setup_periph() {
    let dp = pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    let clocks = rcc.cfgr.sysclk(24.MHz()).freeze();

    LEDS.init(
        gpioc.pc14.into_push_pull_output(),
        gpioc.pc13.into_push_pull_output(),
    );

    let rx = gpioa.pa10.into_alternate();
    let tx = gpioa.pa9.into_alternate();
    let serial1: Serial<
        stm32f4xx_hal::pac::USART1,
        (
            stm32f4xx_hal::gpio::Pin<'A', 9, stm32f4xx_hal::gpio::Alternate<7>>,
            stm32f4xx_hal::gpio::Pin<'A', 10, stm32f4xx_hal::gpio::Alternate<7>>,
        ),
        u8,
    > = dp.USART1.serial((tx, rx), 57_600.bps(), &clocks).unwrap();

    TELEM1.from_serial(serial1.split());
}
