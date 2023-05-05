extern crate alloc;

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
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use alloc::sync::Arc;
use lazy_static::lazy_static;

type led_blue_type = Pin<'E', 3, Output<PushPull>>;
type led_green_type = Pin<'E', 4, Output<PushPull>>;

pub struct USBREF {}

pub struct USB<'a> {
    serial: SerialPort<'a, UsbBus<USB2>>,
    device: UsbDevice<'a, UsbBus<USB2>>,
}

pub struct HALDATA {
    pub led_blue: AtomicPtr<led_blue_type>,
    pub led_green: AtomicPtr<led_green_type>,
    pub telem1: AtomicPtr<Serial<UART7>>,
    pub usb: Option<AtomicPtr<USB<'static>>>,
}
use core::fmt;

// #[macro_export]
// macro_rules! console_print {
//     ($($arg:tt)*) => ($crate::MatekH743::_print(format_args!($($arg)*)));
// }
//
// #[macro_export]
// macro_rules! console_println {
//     () => ($crate::console_print!("\n"));
//     ($($arg:tt)*) => ($crate::console_print!("{}\n", format_args!($($arg)*)));
// }

impl<'a> USB<'a> {
    pub fn print(&mut self, args: fmt::Arguments) {
        let string = alloc::format!("{}", args);
        let buf = string.as_bytes();
        let mut write_offset = 0;
        let count = buf.len();
        while write_offset < count {
            match self.serial.write(&buf[write_offset..count]) {
                Ok(len) if len > 0 => {
                    write_offset += len;
                }
                _ => {}
            }
        }
    }

    pub fn read_polling() {
        todo!()
        // if usb_dev.poll(&mut [serial]) {
        //     let mut buf = [0u8; 64];
        //
        //     match serial.read(&mut buf) {
        //         Ok(count) if count > 0 => {
        //             // Echo back in upper case
        //             for c in buf[0..count].iter_mut() {
        //                 if 0x61 <= *c && *c <= 0x7a {
        //                     *c &= !0x20;
        //                 }
        //             }
        //             let mut write_offset = 0;
        //             while write_offset < count {
        //                 match serial.write(&buf[write_offset..count]) {
        //                     Ok(len) if len > 0 => {
        //                         write_offset += len;
        //                     }
        //                     _ => {}
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
    }

    pub fn poll(&mut self) -> bool {
        self.device.poll(&mut [&mut self.serial])
    }

}


static mut EP_MEMORY: [u32; 1024] = [0; 1024];

impl HALDATA {
    fn new() -> Self {
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


    pub fn take_usb(&self) -> Option<&'static mut USB>  {
        match self.usb.as_ref() {
            Some(ap) => unsafe { ap.load(Relaxed).as_mut() }
            None => None,
        }
    }
}

lazy_static! {
    pub static ref HAL: HALDATA = HALDATA::new();
}
