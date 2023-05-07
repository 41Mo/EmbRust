use core::cell::RefCell;

use crate::alloc::{fmt, string::ToString};
use crate::core::{
    format_args,
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering::Relaxed},
};
use boards::periph::HAL;
use freertos_rust::{CurrentTask, Duration, DurationTicks, Queue, TaskDelay, TaskDelayPeriodic};

#[derive(Copy, Clone)]
enum TasksNum {
    Task1,
    Task2,
    Task3,
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
            TasksNum::Task3 => "EmptyTask",
        };
        write!(
            f,
            "Task {}:\n\t max_free_mem: {} bytes",
            task_name,
            self.min_free_mem * 4
        )
    }
}

static GLOBAL_QUEUE: AtomicPtr<Queue<TasksData>> = AtomicPtr::new(null_mut());
static USB_READY: AtomicBool = AtomicBool::new(false);
static POLL_TICKS: AtomicU32 = AtomicU32::new(0);

pub fn empty_task() {
    let mut last_send_time = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };
    let mut task_delay = TaskDelay::new();
    let mut task_run_count = 0;
    let mut diff_cum = 0;

    let mut last_send_time_queue = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };
    loop {
        USB_READY.store(HAL.usb_poll(), core::sync::atomic::Ordering::SeqCst);

        let now = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };
        diff_cum += now - last_send_time;
        last_send_time = now;
        task_run_count += 1;

        if now - last_send_time_queue > Duration::ms(1000).to_ticks() {
            match unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() } {
                Some(q) => {
                    let min_free_mem = CurrentTask::get_stack_high_water_mark();
                    let _ = q.send(
                        TasksData {
                            num: TasksNum::Task3,
                            min_free_mem,
                        },
                        Duration::ms(2),
                    );
                }
                None => (),
            }
            last_send_time_queue = now;
        };

        if task_run_count >= 100 {
            POLL_TICKS.store(
                diff_cum / task_run_count,
                core::sync::atomic::Ordering::SeqCst,
            );
            task_run_count = 0;
            diff_cum = 0;
        }

        task_delay.delay_until(Duration::ms(5));
    }
}

pub fn blink() {
    let mut task_delay = TaskDelay::new();
    loop {
        let led = HAL.led_green.load(Relaxed);
        match unsafe { led.as_mut() } {
            Some(led) => led.toggle(),
            None => continue,
        }

        match unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() } {
            Some(q) => {
                let min_free_mem = CurrentTask::get_stack_high_water_mark();
                let _ = q.send(
                    TasksData {
                        num: TasksNum::Task1,
                        min_free_mem,
                    },
                    Duration::ms(2),
                );
            }
            None => (),
        }
        task_delay.delay_until(Duration::ms(1000));
    }
}

fn usb_comm() {
    if !USB_READY.load(core::sync::atomic::Ordering::SeqCst) {
        return;
    }

    let q = unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() }.expect("Queue ptr null");

    let _msg = match q.receive(Duration::ms(1)) {
        Ok(data) => data.to_string(),
        Err(e) => {
            let err = match e {
                freertos_rust::FreeRtosError::OutOfMemory => "OOM",
                freertos_rust::FreeRtosError::QueueSendTimeout => "send Timeout",
                freertos_rust::FreeRtosError::QueueReceiveTimeout => return,
                freertos_rust::FreeRtosError::MutexTimeout => "mutexTimeout",
                freertos_rust::FreeRtosError::Timeout => "timeout",
                freertos_rust::FreeRtosError::QueueFull => "queueFull",
                freertos_rust::FreeRtosError::StringConversionError => "string conv err",
                freertos_rust::FreeRtosError::TaskNotFound => "task not found",
                freertos_rust::FreeRtosError::InvalidQueueSize => "invalid size",
                freertos_rust::FreeRtosError::ProcessorHasShutDown => "processor shutdown",
            };
            HAL.usb_print(format_args!("Queue err: {}\n", err));
            return;
        }
    };

    let ticks = POLL_TICKS.load(core::sync::atomic::Ordering::SeqCst);
    let freq = 1000 / Duration::ticks(ticks).to_ms();
    HAL.usb_print(format_args!("{}\n", _msg));
    HAL.usb_print(format_args!("Usb poll freq: {}\n", (freq).to_string()));
}

pub fn console() {
    let mut mq = Queue::<TasksData>::new(5).expect("Unable to create queue");
    GLOBAL_QUEUE.store(&mut mq, Relaxed);
    let mut task_delay = TaskDelay::new();
    let mut last_send_time = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };

    loop {
        usb_comm();
        let now = unsafe { freertos_rust::freertos_rs_xTaskGetTickCount() };
        if now - last_send_time > 1000 {
            let q = unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() }.expect("Queue ptr null");

            match q.send(
                TasksData {
                    num: TasksNum::Task2,
                    min_free_mem: CurrentTask::get_stack_high_water_mark(),
                },
                Duration::ms(2),
            ) {
                Ok(_) => (),
                Err(_) => (),
            }
            last_send_time = now;
        }

        task_delay.delay_until(Duration::ms(200));
    }
}
