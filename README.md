# bake-rs

Note: This project is not ready! it's just an idea.

Better Make written in Rust.

Note: bake can be used for any type of projects (it's not only for rust) see [examples](TODO)

## Table of content

* Basic
    * Init
    * CLI
    * TUI
    * GUI
    * Web
* Error handling
* Build dependencies
* Run other tasks from a task
* Platform specific commands
* Plugin system
* Task param
* Template engine
* Global variables

## Basic

Make a file named 'bakefile.yaml' in root of your project

```yaml
tasks:
    - name: clean
        help-msg: this task removes what you build
        cmd: 
            - rm ./build
    - name: hello
        help-msg: this task says hello
        cmd: 
            - echo hello world
            - echo hello from bake
```

Or a json file named bakefile.json

```json
{ 
    "tasks": [
        {
            "name": "clean",
            "help-msg": "this task removes what you build",
            "cmd": ["rm ./build"]
        },
        {
            "name": "hello",
            "help-msg": "this task says hello",
            "cmd": [
                "echo hello world",
                "echo hello from bake"
            ]
        },
    ]
}
```

Or you can make this file by running:

```sh
bake --setup
```

Now you have many ways to run your tasks:

### CLI

```sh
$ bake show

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

TODO

### Web

With web interface you can run commands remotely and see result.

First you need to configure server in yaml file like:

```yaml
server:
    username: USERNAME
    password: PASSWORD
    default-port: 3001
    public-serve: ./public-path
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
    task:
        name: hello
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

Note: task 'echo 3' will not run

## Build dependencies

Sometimes you need some stuff installed on system to run a command.

for example to run build task you need to have 'rust' installed

check this out:

```yaml
dependencies:
    - name: rust
        check: cargo --version
        install-link: https://www.rust-lang.org/tools/install # install button opens browser and user should manually install it

    - name: clippy
        check: cargo clippy --version
        install-cmd: cargo install clippy # install button will automatically install
```

Now you tasks can depends on dependencies

```yaml
tasks:
    -name: release
        dependencies: 
            - rust
        cmd: 
            - cargo build --release
    - name: check:
        dependencies: 
            - rust
            - clippy
        cmd:
            - cargo check
            - cargo clippy
            - cargo fmt --check
            - cargo test
```

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

## variables and params

### data types

* int
* float
* string
* bool
* enum

### global configs

```yaml
global-configs:
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

### param

```yaml
run:
    params:
        - PORT: 
            type: int
            default: 80
```

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
