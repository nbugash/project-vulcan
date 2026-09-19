# Overview
Vulcan is a desktop IDE with IntelliJ's look, feel, and workflow richness, built to be measurably snappier than every current alternative when editing any language, locally or against a remote machine.

# Requirement
## Functional Requirements
* Should support the following languages:
    * JVM languages
    * Go / Golang
    * Elixir
    * Rust
    * Python
    * Zig
    * C / C++
    * Javascript
* Similar to Intellij or VSCode, this IDE should be extensible by creating custom plugins - languages, themes, keymaps, etc.
* Should be able to do remote development such as the source code is located in an EC2 instance while the client is a low-powered laptop with basic specs
* Should be able to do remote debugging 
    * JVM Languages: Java Platform Debugger Architecture (JPDA) and JDWP (Java Debug Wire Protocol)
    * Go: Delve (dlv)
    * Elixir: [ElixirLS](https://github.com/elixir-lsp/elixir-ls)
    * Rust, Zig, C and C++: Running a debugging server (gdbserver or lldbserver or CodeLLDB)
    * Python: debugpy
    * NodeJs: using the built-in V8 Inspector Protocol, typically secured via an SSH tunnel or port forwarding
* Should have an interactive debugger - breakpoints, set values for variable, call stack, live evaluation
* When debugging, should handle reactive streams written in Java/Scala/Kotlin
* Should have a built-in VCS (Version Control System) such as git - staging, branches, commit, tree-visualizer, diff, merge, etc.
* Should have context-aware autocompletion
* Should have a seemless integration with Claude
* Should handle automated refactoring - rename, extract method, move classes, project-wide
* Should have static analysis, realtime error detection and code smell
* Should handle symbols
* Should display syntax colouring in supported languages
* Should be able to undo and redo, with edits grouped by typing run rather than by individual keystroke
* Should work on linux and macos
* Should have smart code assistance & automation
    * Context-Aware Autocompletion: Goes beyond simple word matching by analyzing types, signatures, and scope to offer intelligent completions.
    * Automated Refactoring: Performs safe, project-wide refactoring (renaming symbols, extracting methods, moving classes) while preserving code integrity
    * Static Analysis & Real-Time Error Detection: Highlights syntax errors, code smells, and potential runtime bugs as you type.
* Should have deep code navigation & search
    * Symbol & Usage Tracking: Allows developers to jump instantly to definitions, implementation classes, or usage locations across massive codebases
    * Global & Fuzzy Search: Enables fast lookup of files, classes, symbols, and settings with quick keystrokes
* Should have integrated build, test, & debug dools
    * Interactive Debugger: Offers breakpoints, variable inspection, call stack navigation, and live expression evaluation.
    * Built-in Test Runners: Executes unit and integration tests directly inside the editor, rendering visual test trees and coverage reports.
    * Build System Integration: Native or plugin-based support for build tools like Gradle, Maven, or Bazel.
* Should have version control & ecosystem
    * Visual Version Control (VCS): Built-in Git tools for staging, branch management, commit visualizers, side-by-side diffing, and merge conflict resolution.
    * Plugin Ecosystem & Customization: Extensibility through plugins to support new languages, cloud services, themes, and keymaps.

## Non-functional requirements:
* Should be near-realtime latency when it comes to remote development (< 10ms of latency)
* Should be able to run on a budget friendly computer with 6CPU core and 8 GB of RAM when doing remote development
* the code structure should adhere to the Hexagonal Architecture
* Should be able to handle large code bases
* Should have automated test suite

# Product Architecture
Every feature in the IDE should be a plugin. The only difference is that these functional requirements listed comes installed by default - similar to Eclipse where its blueprint relies on 3 core pillars: OSGi Bundles, the Extension/Extension Point model, and Lazy Loading. 
Also create the implementation architecture documentations and use mermaid to draw out the diagrams

# Core Plugin Extensibility Hooks
To ensure a third-party developer can build a plugin for a language not yet supported (e.g. Nim, or a proprietary company-specific language), the core API must expose:
- Custom Run Configurations: Let plugins define their own "Run/Debug" dialog boxes (e.g., a plugin could add a "Run via Docker" option).
- Lexer and Highlighter APIs: Expose an API (often based on TextMate grammars or Tree-sitter) so plugins can easily paint syntax colors on the screen.
- Custom Tool Windows: Allow plugins to dock their own UI panels on the left, right, or bottom of the screen (like a database explorer or a specific test runner).
Going with an LSP (Language Server Protocol) architecture is the perfect choice for a remote development setup. It inherently solves the exact hardware imbalance described by cleanly decoupling the "UI/Presentation" from the "Heavy Computation."

# The Core Concept: Decoupling the Brain from the Eyes
Historically, IDEs like Eclipse or older versions of IntelliJ parsed the code, ran the compiler, and rendered the UI all in the same massive application process on your local machine.
* The Language Server Protocol changes this by creating a Client-Server model
* The Client: Acts only as a dumb terminal. It renders text, handles keystrokes, and displays popups.
* The Server (The Language Server on EC2): Acts as the brain. It reads the file system, builds the Abstract Syntax Tree (AST), tracks references, and runs the compiler
## Architecture Diagram: Remote LSP
```
LOCAL MACHINE                           NETWORK                     REMOTE SERVER
[ MacBook Neo (6 CPU, 8GB RAM) ]                                [ EC2 Instance (16 CPU, 64GB RAM) ]
                                                
+--------------------------+                                    +----------------------------------+
|      Your IDE App        |                                    |   Remote File System / Codebase  |
|                          |                                    +----------------------------------+
|  +--------------------+  |                                    |    Language Servers (Daemons)    |
|  |                    |  |   JSON-RPC Messages via SSH/TCP    |                                  |
|  | 1. Text Editor     |  | ---------------------------------> |   * gopls (Golang)               |
|  | 2. File Tree UI    |  |  (e.g., "Give me completions for   |   * rust-analyzer (Rust)         |
|  | 3. LSP Client      |  |         line 15, col 20")          |   * jdtls (Java/JVM)             |
|  |                    |  | <--------------------------------- |                                  |
|  +--------------------+  |  (e.g., "Here is a JSON array of   |   - Parses thousands of files    |
+--------------------------+       functions and variables")    |   - Caches symbols in memory     |
                                                                |   - Executes background linters  |
 ```


# Paradigm-Specific IDE Features
Because the default languages span vastly different paradigms, the plugin APIs must expose UI and debugging hooks that cater to their unique needs

## For Systems Languages (C, C++, Rust)
Native Debugging Integration: You need seamless UI wrappers around GDB or LLDB to inspect memory addresses, pointers, and registers.
Macro Expansion: Rust and C/C++ rely heavily on macros. A good IDE lets a developer click a macro and see the expanded code inline before compilation.
Complex Build Systems: Hooks for CMake (C/C++) and Cargo (Rust) to manage dependencies and build targets natively.

## For JVM Languages (Java, Kotlin, Scala)
Bytecode Decompilation: The ability to click into a compiled .jar file library and read decompiled, readable code.
Heavy Refactoring: JVM developers expect to be able to right-click a class and safely extract an interface, push members down to subclasses, or rename a variable across a million-line monolithic codebase safely.
Deep Build Tool Integration: First-class UI support for Maven and Gradle.

## For Concurrency-Heavy Languages (Golang, Elixir)
Concurrency Visualizers: Standard debuggers step line-by-line. For Go (Goroutines) and Elixir (Erlang VM / BEAM lightweight processes), your debugger needs a thread/process monitor to visualize thousands of concurrent routines without crashing the IDE.
Hot Code Reloading: Elixir developers expect to be able to inject new code into a running application without restarting the server. Your IDE should support live execution hooks.

## For Dynamic / Interpreted Languages (Python)
Environment Management: The IDE must natively understand and allow users to switch between virtual environments (venv, conda, poetry) directly from the status bar.
Interactive REPL: A built-in terminal that can execute highlighted code snippets on the fly.

# References
- [Intellij Community Edition](https://github.com/jetbrains/intellij-community)