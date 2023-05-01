use crate::serial::TELEM1;
use crate::led::LEDS;
use crate::hal::{gpio::*, pac, prelude::*};

pub fn setup_periph() {
    let dp = pac::Peripherals::take().unwrap();
    let pwrcfg = dp.PWR.constrain().freeze();
    let rcc = dp.RCC.constrain();

    let ccdr = rcc
        .sys_ck(200.MHz())
        .pll1_q_ck(200.MHz())
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
