# Running the Project

This project is based on the [risc0-foundry-template](https://github.com/risc0/risc0-foundry-template/).

## Option 1: Run directly on host machine
```bash
cd src
```

## Option 2: Run inside Docker
```bash
docker-compose up --build
docker-compose run --rm risc0-test bash
```

## Build & Test
After setting up using one of the options above, you can:
```bash
# Build the project
cargo build
forge build

# Run tests
cargo test
RISC0_DEV_MODE=false forge test -vvv
```
