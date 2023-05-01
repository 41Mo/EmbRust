use core::sync::atomic::{AtomicPtr, Ordering};
use crate::periph::*;

pub struct Leds<B, G> {
    pub led_blue: AtomicPtr<B>,
    pub led_green: AtomicPtr<G>,
}

impl Leds<LedBlueType, LedGreenType> {
    pub fn init(&self, mut green: LedGreenType, mut blue: LedBlueType) {
        green.set_high();
        blue.set_high();
        self.led_green.store(&mut green, Ordering::Relaxed);
        self.led_blue.store(&mut blue, Ordering::Relaxed);
    }

    pub fn toggle_green(&self) {
        unsafe {
            self.led_green
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .toggle();
        }
    }

    pub fn toggle_blue(&self) {
        unsafe {
            self.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .toggle()
        }
    }

    pub fn set_blue_on(&self) {
        unsafe {
            self.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_low();
        }
    }

    pub fn off(&self) {
        unsafe {
            self.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_high();
            self.led_green
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_high();
        }
    }

    pub fn flasher(&self) {
        unsafe {
            self.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_low();
            self.led_green
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_high();
        }
    }
}
