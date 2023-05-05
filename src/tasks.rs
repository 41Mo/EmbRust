use boards;
use boards::periph::HAL;
use freertos_rust::{CurrentTask, Duration, DurationTicks, Queue};
extern crate alloc;
use crate::TASK_HANDLES;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::format_args;
use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::AtomicPtr;

static GLOBAL_QUEUE: AtomicPtr<Queue<[u8; 255]>> = AtomicPtr::new(null_mut());

struct HeapArray<T> {
    ptr: *mut T,
    len: usize,
    min_size: usize,
}

impl<T> HeapArray<T> {
    fn new(len: usize) -> HeapArray<T> {
            let layout = Layout::from_size_align(len * size_of::<T>(), 8).unwrap();
            let min_size = layout.size();
            let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) } as *mut T;
        Self { ptr, len, min_size }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_ref(&self, idx: usize) -> Option<&T> {
        match idx < self.len() {
            true => unsafe { Some(&*(self.ptr.add(size_of::<T>() * (idx)))) },
            false => None,
        }
    }

    fn as_mut(&self, idx: usize) -> Option<&T> {
        match idx < self.len() {
            true => unsafe { Some(&mut *(self.ptr.add(size_of::<T>() * (idx)))) },
            false => None,
        }
    }

    fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

pub fn blink() {
    loop {
        let led = HAL.led_green.load(core::sync::atomic::Ordering::Relaxed);
        match unsafe { led.as_mut() } {
            Some(led) => led.toggle(),
            None => continue,
        }
        CurrentTask::delay(Duration::ms(1000));
    }
}

fn send_tasks_status(
    last_send_time: u32,
    task_info_arr: &HeapArray<freertos_rust::FreeRtosTaskStatusFfi>,
    usb: &mut boards::periph::USB,
) -> u32 {
    let now = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };

    if now - last_send_time < Duration::ms(1000).to_ticks() {
        return last_send_time;
    };

    // let t1 = task_info_arr[0];
    let ptr = task_info_arr.as_mut_ptr();
    for i in 0..task_info_arr.len {
        // let e = task_info_arr.as_ref(i).unwrap();
        // console_println!(
        //     "Task: {}, stack space left: {}, run time counter: {}",
        //     unsafe { core::ffi::CStr::from_ptr(e.task_name as *const core::ffi::c_char) }
        //         .to_str()
        //         .unwrap_or("unknown"),
        //     e.stack_high_water_mark,
        //     e.run_time_counter
        // );
        // console_println!("Task{}: ", i);
        usb.print(format_args!("Task: {}\n", e.task_number));
    }

    return now;
}

pub fn console() {
    let mut last_send_time = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };
    let n_tasks: freertos_rust::FreeRtosUBaseType =
        unsafe { freertos_rust::freertos_rs_get_number_of_tasks() };
    let mut task_info_arr =
        HeapArray::<freertos_rust::FreeRtosTaskStatusFfi>::new(n_tasks as usize);
    let size = size_of::<freertos_rust::FreeRtosTaskStatusFfi>();
    let usb = HAL.take_usb().expect("Usb doesnt exist");
    loop {
        if !usb.poll() {
            continue;
        }
        get_tasks_status(last_send_time, task_info_arr.as_mut_ptr(), n_tasks);
        last_send_time = send_tasks_status(last_send_time, &task_info_arr, usb);
    }
}
