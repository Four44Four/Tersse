# Purpose
 - Rust based MCP multi agent orchestration customizer TUI thing idk

# Hello-world curses demo

Cross-platform curses TUI using [pancurses](https://crates.io/crates/pancurses) (ncurses on Unix/Linux/macOS, PDCurses in the **current console** on Windows via the `win32` feature).

## Run

```bash
cargo run
```

Release build:

```bash
cargo run --release
```

Activating a button shows its label (`Foo` or `Bar`) under that button for **2 seconds**. Quit with **Q** or **Esc**.

## Platform notes

| OS | Backend | Prerequisites |
|----|---------|---------------|
| Linux | ncurses | `libncurses-dev` (Debian/Ubuntu) or `ncurses` (Fedora/Arch) |
| macOS | ncurses | Xcode CLI tools; ncurses is usually available |
| Unix (BSD, etc.) | ncurses | OS `ncurses` / `ncursesw` development package |
| Windows | PDCurses `win32` (console) | Run from cmd, PowerShell, or Windows Terminal; MSVC or MinGW for build |
| DOS | PDCurses for DOS | Not built by default. Requires a DOS Rust toolchain and linking against a [PDCurses DOS port](https://github.com/wmcbrine/PDCurses); the app uses only portable curses APIs. |

Windows Terminal, cmd, PowerShell, and most modern Unix terminals work. Use a true terminal emulator (not a raw DOS box) for mouse support where available.

# Intended Features
 - TUI interface
    - All user interface related features will be accessible through a lightweight curses interface
    - Can hook into already running background daemon or, if one doesn't already exist, initialize a new background daemon
 - Specialized agent creation
    - Define:
       - Roles/Specialization
       - System prompts
       - Guardrails:
          - What are they allowed to read and/or write to
          - What URL domains are they allowed to access
          - What local tools do they have access to
          - Enforce at both deterministic software level and prompt level
       - Retry policy
       - Task limits:
           - Token usage
           - Duration
 - Specialized agent tracking:
    - See what each agent is doing
    - Stop agents
    - Revert agents' changes
 - Orchestrator agent:
    - Task delegation to specialized agents
    - Summarize current state of agents
    - Automatically stop agents that are violating certain guardrails
 - Agent workflow logging:
    - Define descriptive and hardcoded tags for identifying agent actions
    - Categorize each action that an agent performs with tags
    - Human readable log of all actions that agents have done
       - Significant decisions are emphasized over minor command runs and actions
       - Option to highglight specific action types:
          - Web requests
          - NPM commands
          - Running custom scripts
          - Writing to specific files
 - Workflow hooks:
    - Define custom actions that will be run when certain actions occur:
       - Touching a specific file
       - Making a web request to a specific URL
       - Running a specific command
 - Agent centered VC:
    - Agents take advantage of VC systems to log the diffs of their file system related actions to efficiently revert changes if needed
 - Background persistence:
    - Option to leave it on as a persistent background daemon process on exit
    - Configurable to automatically start as a persistent background daemon process

# Constraints
 - All agent actions must be revertable
 - All agent tasks must be interruptable
 - All agent tasks must be recoverable in case of a crash or outage