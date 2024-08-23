# bake-rs

Note: This project is not ready! it's just an idea.

Better Make written in Rust.

Note: bake can be used for any type of projects (it's not only for rust) see [examples](TODO)

## Table of content

1. Basic
    1. Init
    1. CLI
    1. TUI
    1. GUI
    1. Web
1. Error handling
1. Build dependencies
1. Run other tasks from a task
1. Platform specific commands
1. Plugin system
1. Task param
1. Template engine
1. Global variables

## Basic

Make a file named 'bakefile.yaml' in root of your project

```yaml
tasks:
    - clean:
        help-msg: this task removes what you build
        cmd: 
            - rm ./build
    - hello:
        help-msg: this task says hello
        cmd: 
            - echo hello world
            - echo hello from bake
```

Now you have many ways to run your tasks:

### CLI

```sh
$ bake --show

[1] clean: this task removes what you build
[2] hello: this task says hello

$ 
```

```sh
$ bake clean

task clean -> 'rm ./build' is running
task clean -> 'rm ./build' is done

$ 
```

### TUI

```sh
$ bake

[1] clean: this task removes what you build
[2] hello: this task says hello

Enter task name or [index] to run: 2

task hello -> 'echo hello world' is running
hello world
task hello -> 'echo hello world' is done

task hello -> 'echo hello from bake' is running
hello from bake
task hello -> 'echo hello from bake' is done

[1] clean: this task removes what you build
[2] hello: this task says hello

Enter task name or [index] to run: 
```

### GUI

```sh
bake --gui
```

<img width=300 src="./screenshots/simple_ui.png">

### Web

With web interface you can run commands remotely and see result.

First you need to configure server in yaml file like:

```yaml
server:
    username: USERNAME
    password: PASSWORD
    default-port: 3001
    public-serve: ./public_path
```

You can start server by running:

```sh
bake --start-server --port 3000
```

hint: with web interface you can run some commands on your remote server by clicking a button!

you can set it up on your raspberry pi and use it as a web controller for your project.

Warning: if you are using it over internet make sure its behind an encryption layer (don't leak your username password)

## Error handling

```yaml
tasks:
    hello:
        cmd: 
            - echo 1
            - echo2 # this is an error
            - echo 3
```

```sh
task hello -> 'echo 1' is running
1
task hello -> 'echo 1' is done

task hello -> 'echo2' is running
Command 'echo2' not found
task hello -> 'echo2' failed

bake aborting hello
```

Note: task 'echo 3' will not run.

## Build dependencies

Sometimes you need some stuff installed on system to run a command.

for example to run your build task you need to have 'rust' installed.

check this out:

```yaml
dependencies:
    - rust:
        check: cargo --version
        install-link: https://www.rust-lang.org/tools/install # install button opens browser and user should manually install it

    - clippy:
        check: cargo clippy --version
        install-cmd: cargo install clippy # install button will automatically install
```

Now you tasks can depends on dependencies

```yaml
tasks:
    - release:
        dependencies: 
            - rust # add rust compiler as a dependency for cargo build
        cmd: 
            - cargo build --release
    - check:
        dependencies: 
            - rust
            - clippy
        cmd:
            - cargo check
            - cargo clippy
            - cargo fmt --check
            - cargo test
```

<img width=300 src="./screenshots/dependency_manager.png">

### Run other tasks from a task

```yaml
tasks:
    release:
        dependencies: 
            - rust
        cmd: 
            - tasks.check # run other tasks (before or after your commands)
            - cargo build --release
    check:
        cmd:
            - cargo check
```

## Platform specific commands

Sometimes you need to run different commands on different operating systems:

```yaml
clean:
    cmd: # default  
        - rm ./target
    cmd-windows: 
        - del target
```

## Environment variables

### global env

```yaml
global-env-vars:
    - PORT: 
        default: 80
    - BUILD_MODE: 
        default: debug
```

### param

```yaml
tasks:
    listen:
        env-vars:
            - PORT: 
                default: 80
    cmd:
        - nc -l -p $PORT
```

<img width=300 src="./screenshots/env_vars_and_params.png">

### env validation

supported validation:

1. number
1. integer
1. float
1. bool
1. enum(variation1|variation2|variation3| ...)

```yaml
global-env-vars:
    - PORT: 
        default: 5
        validation: integer
    - build-mode: 
        default: debug
        validation: enum(debug|release)
```

### bake cache

bake can save your env setup on a file

## Plugin system

You can import other people

```yaml
plugins:
    - name: fs
        url: https://github.com/TODO
    - name: git
        url: https://github.com/TODO
    - name: android
        url: https://github.com/TODO
```
