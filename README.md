# TShell

A command tree framework with autocompletion and contextual help based on rustyline.

**Supported Platforms**
* Linux

## Build
This project uses Cargo and Rust stable
```bash
cargo build --release
```

## Example
```rust
#[macro_use]
extern crate tshell;

use tshell::{CommandTree, CommandResult};
use std::collections::HashMap;

fn world(args: HashMap<String, &str>, o_context: &mut Option<Context>, history: &HashMap<String, String>) -> CommandResult<Option<String>> {
    println!("World");
    Ok(None)
}

fn darkness(args: HashMap<String, &str>, o_context: &mut Option<Context>, history: &HashMap<String, String>) -> CommandResult<Option<String>> {
    if let Some(friend) = args.get("friend") {
        println!("Darkness, friend = {}", friend);
    }
    Ok(None)
}

fn main() {
    let mut root = shell_command_tree!{my_cli,
        "MyCLI",
        "0.1.0",
        [
            shell_command_node!{
                cmd: hello,
                txt_help: "Hello Root",
                nodes: [
                    shell_command_node!{
                        cmd: world,
                        txt_help: "World",
                        callback: world
                    },
                    shell_command_node!{
                        cmd: darkness,
                        txt_help: "Darkness",
                        callback: dannkess,
                        args: [friend => true]
                    }
                ]
            }]
        };
        root.run();
}

```
## crates.io
You can use this package in your project by adding the following
to your `Cargo.toml`:

```toml
[dependencies]
tshell = "0.1.0"
```

## Features
 - Command tree structure
 - Command completion
 - Contextual help
 - Command history
 - Context switching

 ## Built in commands

 Command    | Action
 ---------  | ------
 up | Move a level, exit from current context
 top | Move to top context
 exit or quit | Exit the shell
 help | lists all the available commands
 ? | contextual help
 [Object] ? | help for that object


 ## ToDo
  - Take input from a file
  - Output to a file
  
