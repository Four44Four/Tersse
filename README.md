# Purpose
 - Rust based stupid and simple file management TUI library

# Intended Features
 - Buttons
    - Trigger any function on press
 - Multi-option menus
    - Triggers new screen with menu name + options as buttons
       - Menu name displays number of menus active right now
 - Text input fields
    - Newline supported
    - Auto height expand supported
    - Copy, cut, and paste supported
 - Text display fields
    - Can scroll
    - Can update in realtime
 - *Message gutter
    - Can be placed at the top or bottom of screen
    - Option can expand to fit any msg, or be limited to specific height
 - *Help screen
    - Displays all keybinds and any tips needed to use
 - *Keybind screen
    - Displays buttons for rebinding actions
 - *Message screen
    - Realtime message log
 - *File explorer
    - Option: Render diffs made on disk
 - *File renderer
    - Option: Render diffs made on disk

# Constraints
 - When creating a new TUI element:
    - It must be created on the same thread that the TUI was created on
    - Or, it must be created within a queued update for the TUI
       - All queued updates will automatically run in the context of the TUI's thread
    - Violating this will result in internal TUI state variables experiencing race conditions and data corruption