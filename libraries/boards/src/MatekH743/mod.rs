use core::marker::Sync;


use crate::hal::{
    device::UART7,
    gpio::*,
    pac,
    prelude::*,
    rcc::rec::UsbClkSel,
    serial::Serial,
    usb_hs::{UsbBus, USB2},
};
use core::{
    option::{Option, Option::*},
    sync::atomic::{AtomicPtr, Ordering::Relaxed},
};

use lazy_static::lazy_static;
use usb_device::{prelude::*};
use usbd_serial::SerialPort;

extern crate alloc;


use alloc::sync::Arc;



type led_blue_type = Pin<'E', 3, Output<PushPull>>;
type led_green_type = Pin<'E', 4, Output<PushPull>>;

pub struct USBREF {}

pub struct USB<'a> {
    pub serial: SerialPort<'a, UsbBus<USB2>>,
    pub device: UsbDevice<'a, UsbBus<USB2>>,
}

pub struct HALDATA {
    pub led_blue: AtomicPtr<led_blue_type>,
    pub led_green: AtomicPtr<led_green_type>,
    pub telem1: AtomicPtr<Serial<UART7>>,
    pub usb: Option<AtomicPtr<USB<'static>>>,
}
// impl From<NanoSeconds> for Duration {
//     fn from(ns: NanoSeconds) -> Self {
//         Self::from_nanos(ns.0 as u64)
//     }
// }
//
// /// A monotonic nondecreasing timer
// #[derive(Clone, Copy)]
// pub struct MonoTimer {
//     frequency: Hertz,
// }
//
// impl MonoTimer {
//     /// Creates a new `Monotonic` timer
//     pub fn new(mut dwt: DWT, clocks: Clocks) -> Self {
//         dwt.enable_cycle_counter();
//
//         // now the CYCCNT counter can't be stopped or resetted
//         drop(dwt);
//
//         MonoTimer {
//             frequency: clocks.sysclk(),
//         }
//     }
//
//     /// Returns the frequency at which the monotonic timer is operating at
//     pub fn frequency(&self) -> Hertz {
//         self.frequency
//     }
//
//     /// Returns an `Instant` corresponding to "now"
//     pub fn now(&self) -> Instant {
//         Instant {
//             now: DWT::cycle_count(),
//         }
//     }
// }

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

impl HALDATA {
    pub fn new() -> Self {
        let dp = pac::Peripherals::take().unwrap();
        let pwrcfg = dp.PWR.constrain().freeze();
        let rcc = dp.RCC.constrain();
        let mut ccdr = rcc.sys_ck(80.MHz()).freeze(pwrcfg, &dp.SYSCFG);
        let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
        ccdr.peripheral.kernel_usb_clk_mux(UsbClkSel::Hsi48);

        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

        let mut led_blue = gpioe.pe3.into_push_pull_output();
        let mut led_green = gpioe.pe4.into_push_pull_output();
        led_blue.set_high();
        led_green.set_high();

        let rx = gpioe.pe7.into_alternate::<7>();
        let tx = gpioe.pe8.into_alternate::<7>();
        let mut telem1 = dp
            .UART7
            .serial((tx, rx), 57_600.bps(), ccdr.peripheral.UART7, &ccdr.clocks)
            .unwrap();

        let usb = USB2::new(
            dp.OTG2_HS_GLOBAL,
            dp.OTG2_HS_DEVICE,
            dp.OTG2_HS_PWRCLK,
            gpioa.pa11.into_alternate(),
            gpioa.pa12.into_alternate(),
            ccdr.peripheral.USB2OTG,
            &ccdr.clocks,
        );

        let usb_bus = Arc::new(AtomicPtr::new(&mut UsbBus::new(usb, unsafe {
            &mut EP_MEMORY
        })));

        let serial =
            usbd_serial::SerialPort::new(unsafe { usb_bus.load(Relaxed).as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { usb_bus.load(Relaxed).as_ref().unwrap() },
            UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST PORT 2")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

        Self {
            led_blue: AtomicPtr::new(&mut led_blue),
            led_green: AtomicPtr::new(&mut led_green),
            telem1: AtomicPtr::new(&mut telem1),
            usb: {
                Some(AtomicPtr::new(&mut USB {
                    serial,
                    device: usb_dev,
                }))
            },
        }
    }
}

unsafe impl Sync for HALDATA {}

lazy_static! {
    pub static ref HAL: HALDATA = HALDATA::new();
}
