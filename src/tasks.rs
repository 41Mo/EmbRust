use boards;
use boards::periph::{setup_tim2_callback, timestamp, HAL};
use freertos_rust::{CurrentTask, Duration, DurationTicks, FreeRtosSchedulerState, Queue};
extern crate alloc;
use crate::alloc::string::ToString;
use crate::TASK_HANDLES;
use alloc::boxed::Box;
use alloc::fmt::Arguments;
use alloc::string::String;
use core::alloc::Layout;
use core::borrow::{Borrow, BorrowMut};
use core::cell::RefCell;
use core::format_args;
use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::Ordering::Relaxed;
use core::sync::atomic::{AtomicPtr, AtomicU32};
use alloc::fmt;

#[derive(Copy, Clone)]
enum TasksNum {
   Task1,
   Task2
}

#[derive(Copy, Clone)]
struct TasksData {
    num: TasksNum,
    min_free_mem: u32,
}

impl fmt::Display for TasksData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let task_name = match self.num {
            TasksNum::Task1 => "Blinky",
            TasksNum::Task2 => "Usb",
        };
        write!(f, "Task {}:\n\t mem: {}", task_name, self.min_free_mem)
    }
}

static GLOBAL_QUEUE: AtomicPtr<Queue<TasksData>> = AtomicPtr::new(null_mut());

static TV: AtomicU32 = AtomicU32::new(0);

pub fn blink() {
    loop {
        let led = HAL.led_green.load(Relaxed);
        match unsafe { led.as_mut() } {
            Some(led) => led.toggle(),
            None => continue,
        }

        match unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() } {
            Some(q) => {
                let min_free_mem = unsafe { CurrentTask::get_stack_high_water_mark() };
                let _ = q.send(TasksData { num: TasksNum::Task1, min_free_mem }, Duration::ms(1000));
            }
            None => (),
        }
        CurrentTask::delay(Duration::ms(1000));
    }
}

pub fn console() {
    let mut last_send_time = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };
    let usb = HAL.take_usb().expect("Usb doesnt exist");
    let mut mq = Queue::<TasksData>::new(5).expect("Unable to create queue");
    GLOBAL_QUEUE.store(&mut mq, Relaxed);
    loop {
        if !usb.poll() {
            continue;
        }
        let now = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };

        if now - last_send_time < Duration::ms(1000).to_ticks() {
            continue;
        };

        let q = unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() }.expect("Queue ptr null");

        let _msg = match q.receive(Duration::ms(20)) {
            Ok(data) => data.to_string(),
            Err(e) => {
                let err = match e {
                    freertos_rust::FreeRtosError::OutOfMemory => "OOM",
                    freertos_rust::FreeRtosError::QueueSendTimeout => "send Timeout",
                    freertos_rust::FreeRtosError::QueueReceiveTimeout => "recvTmeout",
                    freertos_rust::FreeRtosError::MutexTimeout => "mutexTimeout",
                    freertos_rust::FreeRtosError::Timeout => "timeout",
                    freertos_rust::FreeRtosError::QueueFull => "queueFull",
                    freertos_rust::FreeRtosError::StringConversionError => "string conv err",
                    freertos_rust::FreeRtosError::TaskNotFound => "task not found",
                    freertos_rust::FreeRtosError::InvalidQueueSize => "invalid size",
                    freertos_rust::FreeRtosError::ProcessorHasShutDown => "processor shutdown",
                };
                err.to_string()
            }
        };

        let cinfo = TasksData{num: TasksNum::Task2, min_free_mem: unsafe { CurrentTask::get_stack_high_water_mark() }};

        usb.print(format_args!("{}\n{}\n", _msg, cinfo.to_string()));

        last_send_time = now;
    }
}
