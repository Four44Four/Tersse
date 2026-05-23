# Purpose
 - This is a Rust crate for a TUI library for file management

# Rules
 - Text that is to be written into README.md for the purposes of documentation or otherwise must be written in AGENT_DUMP.md
    - Do not write text into README.md
 - Implement new features using pure functions to handle the critical functionality logic whenever possible
    - Whenever pure functionality logic changes:
       - Modify existing test targetting that functionality or create a new unit test in ./tests
 - Refer to files in directory ./specifications for technical specifications on components and features 
 - Do not add any features that allow for scheduling based on UI event loop