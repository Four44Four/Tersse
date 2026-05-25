# Run docker memory test
## Build
 - `docker build -t tersse-valgrind .`
## Run
 - `docker run --rm tersse-valgrind`
## Run with args passed
 - `docker run --rm tersse-valgrind -- --nocapture`
 - `docker run --rm tersse-valgrind pure_button`

# Run examples
 - `cargo run --bin <example-name>`
 - Example names can be found in ./Cargo.toml
    - EXAMPLE: basic_example