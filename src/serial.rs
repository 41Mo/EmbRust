use nb::block;
use crate::hal::{
    prelude::*,
    serial::{Rx, Tx},
    stm32::*,
};
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct UartType<T> {
    tx: AtomicPtr<Tx<T>>,
    rx: AtomicPtr<Rx<T>>,
}

pub static TELEM1: UartType<UART7> = UartType {
    tx: AtomicPtr::new(core::ptr::null_mut()),
    rx: AtomicPtr::new(core::ptr::null_mut()),
};

macro_rules! uart_ops {
    ($usartX:ident) => {
        impl UartType<$usartX> {
            pub fn from_serial(&self, mut serial: (Tx<$usartX>, Rx<$usartX>)) {
                self.rx.store(&mut serial.1, Ordering::Relaxed);
                self.tx.store(&mut serial.0, Ordering::Relaxed);
            }
            pub fn write(&self, byte: u8) {
                unsafe {
                    block!(self
                        .tx
                        .load(Ordering::Relaxed)
                        .as_mut()
                        .unwrap()
                        .write(byte))
                    .unwrap()
                }
            }
            pub fn read(&self) -> u8 {
                unsafe { block!(self.rx.load(Ordering::Relaxed).as_mut().unwrap().read()).unwrap() }
            }
        }
    };
}

uart_ops! {
    UART7
}
