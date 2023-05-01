
use boards::periph::{HAL, write_serial};
use freertos_rust::{CurrentTask, Duration};

pub fn default_task() {
    loop {}
}

pub fn telem1rw() {
    loop {
        // let byte = TELEM1.read();
        // TELEM1.write(byte);
    }
}

pub fn blink() {
    let led_green = unsafe { HAL.take_led_green().unwrap() };
    loop {
        // LEDS.toggle_green();
        led_green.toggle();
        CurrentTask::delay(Duration::ms(1000));
    }
}

pub fn usb_read() {
    let usb_bus = unsafe { HAL.take_usb_bus().unwrap() };
    let mut usb_serial = usbd_serial::SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST PORT 2")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    //
    // loop {
    //     if usb_dev.poll(&mut [&mut usb_serial]) {
    //         match usb_serial.read(&mut buf) {
    //             Ok(count) if count > 0 => {
    //                 // Write to both ports
    //                 write_serial(&mut usb_serial, &buf, count);
    //             }
    //             _ => {}
    //         }
    //     }
    // }
}
