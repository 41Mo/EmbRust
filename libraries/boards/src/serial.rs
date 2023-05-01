use crate::hal::{
    serial::{Rx, Tx},
};
use core::sync::atomic::AtomicPtr;

pub struct UartType<T> {
    pub tx: AtomicPtr<Tx<T>>,
    pub rx: AtomicPtr<Rx<T>>,
}

#[macro_export]
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
