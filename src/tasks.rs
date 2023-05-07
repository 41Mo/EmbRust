use boards::periph::HAL;
use freertos_rust::{CurrentTask, Duration, DurationTicks, Queue};
use crate::alloc::{fmt, string::ToString};
use crate::core::{
    format_args,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering::Relaxed},
};

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
            TasksNum::Task3 => "EmptyTask"
        };
        write!(f, "Task {}:\n\t max_free_mem: {} bytes", task_name, self.min_free_mem*4)
    }
}

static GLOBAL_QUEUE: AtomicPtr<Queue<TasksData>> = AtomicPtr::new(null_mut());

pub fn empty_task() {
    loop {
        match unsafe { GLOBAL_QUEUE.load(Relaxed).as_ref() } {
            Some(q) => {
                let min_free_mem = CurrentTask::get_stack_high_water_mark();
                let _ = q.send(
                    TasksData {
                        num: TasksNum::Task3,
                        min_free_mem,
                    },
                    Duration::ms(1000),
                );
            }
            None => (),
        }
        CurrentTask::delay(Duration::ms(1000));
    }
}

pub fn blink() {
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
                    Duration::ms(1000),
                );
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

        let cinfo = TasksData {
            num: TasksNum::Task2,
            min_free_mem: CurrentTask::get_stack_high_water_mark(),
        };

        usb.print(format_args!("{}\n{}\n", _msg, cinfo.to_string()));

        last_send_time = now;
    }
}
