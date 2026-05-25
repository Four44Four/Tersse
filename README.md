# Purpose
 - Rust based stupid and simple TUI library

# Intended Features
 - Buttons
    - Trigger any function on press
 - Text input fields
    - Newline supported
    - Auto height expand supported
    - Copy, cut, and paste supported
 - Text display fields
    - Can scroll
    - Can update in realtime
 - *Miniscreens
    - Allow for various elements to exist in a Screen-like rectangle that clips elements outside of its bounds
 - *Screens
    - Allow for different sets of elements to replace all current elements 
 - Message gutter
    - Can be placed at the top or bottom of screen
    - Option can expand to fit any msg, or be limited to specific height
 - *Message screen
    - Realtime message log
 - *Help screen
    - Displays all keybinds and any tips needed to use
 - *Keybind screen
    - Displays buttons for rebinding actions

# Constraints
 - When creating a new TUI element:
    - It must be created on the same thread that the TUI was created on
    - Or, it must be created within a queued update for the TUI
       - All queued updates will automatically run in the context of the TUI's thread
    - Violating this will result in internal TUI state variables experiencing race conditions and data corruption