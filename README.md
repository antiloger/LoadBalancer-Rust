# Simple_LoadBalancer-Rust

## Overview

This is a simple round-robin load balancer implemented in Rust. It distributes incoming network requests to a pool of backend servers in a circular order, ensuring even load distribution.

## Features

- Round-robin algorithm for load balancing
- Configuration via JSON file
- Lightweight and efficient Rust implementation

## Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/antiloger/Simple_LoadBalancer-Rust.git
    cd Simple_LoadBalancer-Rust
    ```

2. Build the project using Cargo:
    ```sh
    cargo build --release
    ```

## Usage

1. Edit the `server.json` file to add backend servers.

2. Run the load balancer with:
    ```sh
    RUST_LOG=info cargo run --release
    ```
![Screenshot 2024-07-03 193535](https://github.com/antiloger/Simple_LoadBalancer-Rust/assets/114112572/4afa6d32-0a16-4a3a-be3e-2d327a031647)

