# Purpose
 - This is a Rust crate for a simple TUI library

# Rules
 - Text that is to be written into README.md for the purposes of documentation or otherwise must be written in AGENT_DUMP.md
    - Do not write text into README.md
 - Implement new features using pure functions to handle the critical functionality logic whenever possible
    - Whenever pure functionality logic changes:
       - Modify existing test targetting that functionality or create a new unit test in ./tests
 - Refer to files in directory ./specifications for technical specifications on components and features 
 - Do not add any features that allow for scheduling based on UI event loop
 - Whenever new tests are implemented or tests are modified:
    - Run the Dockerfile to test all tests for memory leaks
       - Don't run the examples fuzzing
    - If any are identified:
       - Fix them without compromising functionality or specifications
 - If a function needs to be exposed as API to be used in a test and solely in a test:
    - Move it to a dedicated file/directory to avoid cluttering implementation files