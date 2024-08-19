# bake-rs
Better Make written in Rust.

Note: bake can be used for any type of projects (it's not only for rust) see [examples](TODO)

Why Bake?

- Different ways to use (CLI, TUI, GUI, Web, VS-Code extension)
- Remote command execution over web UI (see [example](TODO))
- Cross platform commands (linux, windows and mac)
- Variables and params
- Build dependencies management
- Template engine
- Form Generator
- Plugin system

## Basic

```yaml
dependencies: # build dependencies
    rust:
        check: cargo --version
        install-link: https://www.rust-lang.org/tools/install # install button opens browser and user should manually install it

    clippy:
        check: cargo clippy --version
        install-cmd: cargo install clippy # install button will automatically install

tasks:

    clean:
        output: silent # run command in background
        cmd: 
            - rm ./target

    hello: 
        output: inline # shows command in UI (for smaller outputs)
        cmd: 
            - echo hello

    release:
        output: terminal # opens a new terminal (for longer outputs)
        dependencies: 
            - rust
        cmd: 
            - tasks.check # run other tasks (before or after your commands)
            - cargo build --release

    check:
        output: terminal
        dependencies: 
            - rust
            - clippy
        cmd:
            - cargo check
            - cargo clippy
            - cargo fmt --check
            - cargo test
```

## Cross platform

Sometimes you need to run different commands on different operating systems:

```yaml
clean:
    output: silent 
    cmd: # default  
        - rm ./target
    cmd-windows: 
        - del target

```

## Template engine

Config.json
```
{
    "ip_addr": "127.0.0.1",
    "port": 80,
}
```

Config.json.baketemplate
```
{
    "ip_addr": "127.0.0.1",
    "port": {{PORT}},
}
```

### global variables

data types:
- int
- float
- string
- bool
- enum

```yaml
GLOBAL_VARS:
    IP_ADDR: 
        type: string
        default: 127.0.0.1
    PORT: 
        type: int
        default: 80
    BUILD_MODE: 
        type: debug|release  # enum type
        default: debug
```

### task param

```yaml
run:
    params:
        - PORT: 
            type: int
            default: 80

```

# Plugin system

You can import other people

```yaml
plugins:
    - url: https://github.com/TODO
      name: fs
    - url: https://github.com/TODO
      name: git
    - url: https://github.com/TODO
      name: android
```