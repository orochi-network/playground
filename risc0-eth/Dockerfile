FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    git \
    libssl-dev \
    pkg-config

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
# Verify cargo is available
RUN cargo --version

# Install Foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:${PATH}"
RUN foundryup

# Install Foundry & Risc0VM
# RUN curl -L https://foundry.paradigm.xyz | bash
RUN curl -L https://risczero.com/install | bash
# add rzup to path
ENV PATH="/root/.risc0/bin:${PATH}"
RUN rzup install
RUN cargo risczero --version

# Set working directory
WORKDIR /app/src

# # Build the repo during image creation
# RUN git submodule update --init
# RUN cargo build
# RUN forge build

# Optional: Run tests during build (uncomment if desired)
# RUN cargo test
# RUN RISC0_DEV_MODE=false forge test -vvv

# Default command (can be overridden in docker-compose)
# CMD ["bash", "-c", "RISC0_DEV_MODE=false forge test -vvvvv"]