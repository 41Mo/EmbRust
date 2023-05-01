use usb_device::class_prelude::UsbBusAllocator;

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
    option::Option,
    result::Result::Ok,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering::Relaxed},
};

type led_blue_type = Pin<'E', 3, Output<PushPull>>;
type led_green_type = Pin<'E', 4, Output<PushPull>>;

pub struct HALDATA {
    led_blue: led_blue_type,
    led_green: led_green_type,
    telem1: Serial<UART7>,
    // pub usb: USB2,
    pub usb_bus: UsbBusAllocator<UsbBus<USB2>>,
}

pub struct LHAL {
    led_blue_ptr: AtomicPtr<Pin<'E', 3, Output<PushPull>>>,
    led_green_ptr: AtomicPtr<Pin<'E', 4, Output<PushPull>>>,
    telem_ptr: AtomicPtr<Serial<UART7>>,
    usb_ptr: AtomicPtr<USB2>,
    hal_data_ptr: AtomicPtr<HALDATA>,
    usb_bus_ptr: AtomicPtr<UsbBusAllocator<UsbBus<USB2>>>,
}

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

impl HALDATA {
    pub fn setup() -> HALDATA {
        let dp = pac::Peripherals::take().unwrap();
        let pwrcfg = dp.PWR.constrain().freeze();
        let rcc = dp.RCC.constrain();
        let mut ccdr = rcc.sys_ck(120.MHz()).freeze(pwrcfg, &dp.SYSCFG);
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
        let telem1 = dp
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

        let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

        // let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });
        // let usb_serial = usbd_serial::SerialPort::new(&usb_bus);
        // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        //     .manufacturer("Fake company")
        //     .product("Serial port")
        //     .serial_number("TEST PORT 2")
        //     .device_class(usbd_serial::USB_CLASS_CDC);

        HALDATA {
            led_blue,
            led_green,
            telem1,
            usb_bus,
        }
    }
}

pub static mut HAL: LHAL = LHAL {
    led_blue_ptr: AtomicPtr::new(null_mut()),
    led_green_ptr: AtomicPtr::new(null_mut()),
    telem_ptr: AtomicPtr::new(null_mut()),
    usb_ptr: AtomicPtr::new(null_mut()),
    hal_data_ptr: AtomicPtr::new(null_mut()),
    usb_bus_ptr: AtomicPtr::new(null_mut()),
};

impl LHAL {
    pub fn freeze(&mut self, haldata: &mut HALDATA) {
        self.led_green_ptr.store(&mut haldata.led_green, Relaxed);
        self.led_green_ptr.store(&mut haldata.led_green, Relaxed);
        self.telem_ptr.store(&mut haldata.telem1, Relaxed);
        // self.usb_ptr.store(&mut haldata.usb, Relaxed);
        // self.hal_data_ptr.store(haldata, Relaxed);
        self.usb_bus_ptr.store(&mut haldata.usb_bus, Relaxed);
    }

    pub fn take_led_blue(&self) -> Option<&mut Pin<'E', 3, Output>> {
        unsafe { self.led_blue_ptr.load(Relaxed).as_mut() }
    }

    pub fn take_led_green(&self) -> Option<&mut Pin<'E', 4, Output>> {
        unsafe { self.led_green_ptr.load(Relaxed).as_mut() }
    }

    pub fn take_telem1(&self) -> Option<&mut Serial<UART7>> {
        unsafe { self.telem_ptr.load(Relaxed).as_mut() }
    }

    pub fn take_usb_bus(&self) -> Option<&mut UsbBusAllocator<UsbBus<USB2>>> {
        unsafe { self.usb_bus_ptr.load(Relaxed).as_mut() }
    }

    pub fn take_haldata(&self) -> Option<&mut HALDATA> {
        unsafe { self.hal_data_ptr.load(Relaxed).as_mut() }
    }
}

pub fn write_serial<P: usb_device::bus::UsbBus>(
    serial: &mut usbd_serial::SerialPort<P>,
    buf: &[u8],
    count: usize,
) {
    if serial.rts() {
        let mut write_offset = 0;
        while write_offset < count {
            match serial.write(&buf[write_offset..count]) {
                Ok(len) if len > 0 => {
                    write_offset += len;
                }
                _ => {}
            }
        }
    }
}
