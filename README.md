# FrtosRs

FrtosRs is a project that aims to explore the usage of the FreeRTOS-rust crate with an embedded application. The project uses the STM32H743VIT6 microcontroller on the MatekH743Wing development board. The project focuses on implementing a FreeRTOS queue, USB communication, and the Rust HAL crate.

## Technologies Used
- [FreeRTOS-rust](https://crates.io/crates/freertos-rust) crate
- [Rust HAL](https://github.com/stm32-rs/stm32h7xx-hal) crate
- STM32H743VIT6 mcu
- [MatekH743Wing](http://www.mateksys.com/?portfolio=h743-wing-v2#tab-id-6) board

## Features
- Implements a FreeRTOS queue for task synchronization
- Sends task status information (such as maximum free stack space) over USB communication
- Blinks green LED on the MatekH743Wing board
- Uses the Rust HAL crate for hardware abstraction

## Getting Started
To get started with this project, you will need to have the following:
- Rust and arm-none-eabi-gcc installed on your machine
- An STM32H743VIT6 microcontroller
- A MatekH743Wing development board

To build the project, follow these steps:
1. Clone the repository using `git clone`.
2. Change into the project directory using `cd frtosrs`.
3. Check that arm-none-eabi-gcc installed `arm-none-eabi-gcc -v`
3. Build the project using `cargo build`.
4. Flash the resulting binary (found at `target/thumbv7em-none-eabihf/release/frtosrs`) to your STM32H743VIT6 microcontroller. Simpliest way to do this is with `cargo embed`

To run the project, simply connect the MatekH743Wing board to your computer and open a serial terminal (such as CuteCom, PuTTY or TeraTerm). The project will start automatically upon flashing the binary to your mcu.

