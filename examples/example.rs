#[macro_use]
extern crate tshell;

use tshell::{CommandTree, CommandResult};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Context {
    memory: Option<String>
}

fn world(args: HashMap<String, &str>, o_context: &mut Option<Context>, history: &HashMap<String, String>) -> CommandResult<Option<String>> {
    if let Some(context) = history.get("context") {
        println!("Context: {}", context);
    }
    println!("World");
    Ok(None)
}

fn darkness(args: HashMap<String, &str>, o_context: &mut Option<Context>, history: &HashMap<String, String>) -> CommandResult<Option<String>> {
    if let Some(friend) = args.get("friend") {
        println!("Darkness, friend = {}", friend);
    }
    else {
        println!("friend argument is required!");
    }
    Ok(None)
}

fn new_context(args: HashMap<String, &str>, o_context: &mut Option<Context>, history: &HashMap<String, String>) -> CommandResult<Option<String>> {
    if let Some(context) = args.get("context") {
        Ok(Some(format!("context:{}", context)))
    }
    else {
        println!("context argument is required!");
        Ok(None)
    }
}


fn main() {
    let mut context = Context::default();
    let mut o_context = Some(context);
    let mut root = shell_command_tree!{my_cli,
        "MyCLI",
        "0.1.0",
        o_context.unwrap(),
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
                        callback: darkness,
                        args: [friend => true]
                    }
                ]
            },
            shell_command_node!{
                cmd: context,
                txt_help: "Hello Root",
                callback: new_context,
                args: [context => true],
                nodes: [
                    shell_command_node!{
                        cmd: world,
                        txt_help: "World",
                        callback: world
                    },
                    shell_command_node!{
                        cmd: darkness,
                        txt_help: "Darkness",
                        callback: darkness,
                        args: [friend => true]
                    }
                ]
            }]
        };
        root.run();
}
