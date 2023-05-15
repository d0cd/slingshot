<h1 align="center">slingshot</h1>

#### `slingshot` has been archived. Please use [snarkOS](https://github.com/AleoHQ/snarkOS) and the associated Developer CLI to run a beacon node for local developement.

# Archive

Slingshot is a lightweight CLI for deploying programs and executing transactions on Aleo.

## Table of Contents

* [1. Overview](#1-overview)
* [2. Build Guide](#2-build-guide)
* [3. Usage Guide](#3-usage-guide)

## 1. Overview

For more information on Aleo, visit [Welcome to Aleo](https://developer.aleo.org/overview/) to get started.

## 2. Build Guide

### 2.1 Install Rust

We recommend installing Rust using [rustup](https://www.rustup.rs/). You can install `rustup` as follows:

- macOS or Linux:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- Windows (64-bit):

  Download the [Windows 64-bit executable](https://win.rustup.rs/x86_64) or
  [Windows 32-bit executable](https://win.rustup.rs/i686) and follow the on-screen instructions.

### 2.2 Build from Source Code

We recommend installing `slingshot` this way. In your terminal, run:

```bash
# Download the source code
git clone https://github.com/d0cd/slingshot.git

# Enter the 'slingshot' directory
cd slingshot

# Install 'slingshot'
cargo install --path .
```

Now to use `slingshot`, in your terminal, run:
```bash
slingshot
```

## 3. Usage Guide

### 3.1 Starting a development node
```
slingshot node start --key <PRIVATE_KEY>
```

### 3.2 Pour from faucet
```
slingshot pour <ADDRESS> <AMOUNT>
```

### 3.2 Deploying a program
```
slingshot deploy --path <PATH_TO_DIR> 
```

### 3.3 Executing a program 
```
slingshot execute <PROGRAM_NAME> <FUNCTION_NAME> <INPUTS>
```
