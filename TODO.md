# Sprint 1
 - Centralize keyboard Tokio runtime with example/basic Tokio runtime
    - UI has a single Tokio runtime that can be used to schedule other bg tasks ?
 - Use aHash for element storage hashing
 - Ensure that only the thread that started the application can update the UI
    - (all updates to UI MUST be synchronized to original UI instance's thread)
    - Write tests for this behavior
 - Message gutter
 - Message screen
 - Help screen
 - Make a new example that is even more basic than basic
 - dockerization in debian container with valgrind
    - run valgrind memory tests inside of container
 - prepare for uploading to crate.io

# Sprint 2
 - Multi-option menu triggering screen
    - Stackable
    - Put on basic example as "Baz" to the right of Foo button with a 3 character gap
       - Shows options "Qux", "Quux", "Plugh", "Xyzzy"
       - Qux -> leads to another menu with options "Swizzle" and "Swazzle"
       - All other buttons lead to a random string of <button's name in all uppercase> + 10 characters being printed to message gutter

# Sprint 3
 - File specific stuff