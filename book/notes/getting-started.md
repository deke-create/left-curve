# Getting Started with Grug

Welcome to the Grug getting started guide! This guide will help you set up your environment, install the necessary tools, and provide some basic usage examples to get you started with building on Grug.

## Prerequisites

Before you begin, make sure you have the following prerequisites installed:

- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Just](https://just.systems/man/en/)
- [Docker](https://docs.docker.com/engine/install/)
- [pnpm](https://pnpm.io/)

## Installation

Follow these steps to install the necessary tools and set up your environment:

1. Install Rust and the `wasm32-unknown-unknown` target:

    ```shell
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustup target add wasm32-unknown-unknown
    ```

2. Install Just:

    ```shell
    curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash
    ```

3. Install Docker by following the instructions for your operating system on the [Docker website](https://docs.docker.com/engine/install/).

4. Install pnpm:

    ```shell
    npm install -g pnpm
    ```

## Basic Usage

Once you have the prerequisites installed, you can start using Grug. Here are some basic commands to get you started:

1. Install the `grug` command line software:

    ```shell
    just install
    ```

2. Run tests:

    ```shell
    just test
    ```

3. Lint the code:

    ```shell
    just lint
    ```

4. Compile and optimize smart contracts:

    ```shell
    just optimize
    ```

## Additional Resources

For more information and detailed documentation, check out the following resources:

- [Grug GitHub Repository](https://github.com/left-curve/grug)
- [Grug Whitepaper](https://leftcurve.software/grug.html)
- [Join our Discord server](https://discord.gg/NAjWt8FQcs)

Happy building on Grug!
