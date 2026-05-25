# Run docker memory test
## Build
- `docker build -t tersse-valgrind .`
## Run
- Tests (default): `docker run --rm tersse-valgrind tests`
- Examples fuzz: `docker run --rm tersse-valgrind examples`

# Run examples
 - `cargo run --bin <example-name>`
 - Example names can be found in ./Cargo.toml
    - EXAMPLE: basic_example