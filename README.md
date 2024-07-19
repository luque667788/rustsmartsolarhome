# Smart Home Frontend (Built with Rust/WASM using Leptos Framework)

This is my smart home frontend developed with Rust (WASM), utilizing the [Leptos](https://leptos.dev/) framework.

The page is served with hydration (SSR and CSR) which is seamlessly implemented out of the box with Leptos. It communicates via MQTT (and some bits of usual HTTP) with an ESP32. 

If there are any questions, feel free to contact me by email: luquemendonca@gmail.com

**Warning**: This project is heavily under development and far from completion. It is important to note that development authentication is only rudimentarily implemented with hard-coded credentials. Also, error handling has only a rudimentary implemention. The code, in general, is messy and lacks proper comments/documentation. There are likely many other security issues with the site which will be addressed later.

## Purpose
This project is designed for on-grid solar grid applications where energy consumption occurs when power is generated to avoid energy production excess. It is intended to work with any time-dependent devices relying on energy to operate, needing to run at specific times of the day.

One of the most useful cases in my specific home situation is a pump for the pool. It needs to run for at least 4 hours per day and consumes a significant amount of energy. Thus, it would be beneficial for it to turn on automatically for only 4 hours a day when the solar panels are producing energy. Additionally, I need the ability to control it remotely - turning it on or off, setting parameters, and toggling between automatic and manual modes. This is where this project comes in. There is an ESP32 that handles turning on the pool pump and controlling its operation time. This specific codebase repository is the "frontend", more specifically the user interface for the entire project.

## Hardware Conditions

It is worth noting that there is also a substantial codebase written in C++ which manages the ESP32 microcontroller part that is currently private but in the future will also be made public.

One thing to consider is that the ESP32´s operating conditions are uncertain. It may be offline due to energy shortages, a memory bug in its code, or any other issue. Therefore, this code needs to handle retaining its last values before it went offline and send them to the ESP32 when it comes back online.

## Deploy
This project can run on docker and everything compiles to a single executable for frontend and backend

