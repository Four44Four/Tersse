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

# Tests
 - `cargo test --features test-api`

# Features
 - `test-api`: exposes `tersse::test_api` and `tersse::pure` for integration tests (not for library users)
 - `debug_should_draw_do_delay`: draw a debug rectangle for a specified amt of time whenever an element is drawn to show when/where element drawing happens