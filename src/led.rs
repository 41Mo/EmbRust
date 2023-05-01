use crate::hal::gpio::*;
use core::sync::atomic::{AtomicPtr, Ordering};

pub type LedBlueType = Pin<'E', 3, Output<PushPull>>;
pub type LedGreenType = Pin<'E', 4, Output<PushPull>>;

pub struct Leds<B, G> {
    pub led_blue: AtomicPtr<B>,
    pub led_green: AtomicPtr<G>,
}

pub static LEDS: Leds<LedBlueType, LedGreenType> = Leds {
    led_blue: AtomicPtr::new(core::ptr::null_mut()),
    led_green: AtomicPtr::new(core::ptr::null_mut()),
};

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
            LEDS.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .toggle()
        }
    }

    pub fn set_blue_on(&self) {
        unsafe {
            LEDS.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_low();
        }
    }

    pub fn off(&self) {
        unsafe {
            LEDS.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_high();
            LEDS.led_green
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_high();
        }
    }

    pub fn flasher(&self) {
        unsafe {
            LEDS.led_blue
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_low();
            LEDS.led_green
                .load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
                .set_high();
        }
    }
}
