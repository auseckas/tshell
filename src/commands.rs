use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use completion::TxCompleter;
use std::sync::{Arc, Mutex};

pub type CommandResult<R> = Result<R, String>;
pub type CallBack<T> = Fn(HashMap<String, &str>, &mut Option<T>, &HashMap<String, String>) -> CommandResult<Option<String>>;

pub struct Node<T> {
    pub cmd: String,
    hidden: bool,
    help: String,
    args: Option<Vec<(String, bool)>>,
    sub_nodes: Option<Vec<Node<T>>>,
    pub callback: Option<Box<CallBack<T>>>
}

impl <T>Node<T> {
    pub fn new(cmd: &str,  conditional: Option<&str>, help: &str, callback: Option<Box<CallBack<T>>>) -> Node<T> {

        let hidden = if let Some(c) = conditional {
            match env::var(&c){
                Ok(_) => false,
                Err(_) => true
            }
        } else {
            false
        };

        Node {
            cmd: cmd.to_owned(),
            hidden: hidden,
            help: help.to_owned(),
            args: None,
            sub_nodes: None,
            callback: callback
        }
    }

    pub fn add_node(&mut self, node: Node<T>) {
        if let Some(ref mut nodes) = self.sub_nodes {
            nodes.push(node)
        }
        else {
            self.sub_nodes = Some(vec![node])
        }
    }

    pub fn add_arg(&mut self, arg: &str, required: bool) {
        if let Some(ref mut args) = self.args {
            args.push((arg.to_owned(), required));
        }
        else {
            let array = vec![(arg.to_owned(), required)];
            self.args= Some(array);
        }
    }

    pub fn find(&self, cmd: &str) -> Option<&Node<T>> {
        debug!("Find: {}", cmd);
        if cmd == "?" {
            self.print_help(1);
            return None;
        }
        if let Some(ref nodes) = self.sub_nodes {
            for node in nodes.iter() {
                if node.cmd == cmd && !node.hidden{
                    return Some(node)
                }
            }
        }
        None
    }

    pub fn get_suggestions(&self, levels: Vec<&str>, idx: usize, _nr_required: usize) -> Option<Vec<&str>> {
        let mut results: Vec<&str> = Vec::new();

        if let Some(ref nodes) = self.sub_nodes {
            debug!("Matching: levels: {:?}, current: {:?}", levels, levels.get(idx));
            let current = levels.get(idx);
            let parent = match idx {
                0 => None,
                _ => levels.get(idx - 1)
            };
            if let Some(name) = current {
                if name.is_empty() {
                    return Some(nodes.iter().map(|n| n.cmd.as_str()).collect());
                }
                for node in nodes {
                    if node.cmd.starts_with(levels[idx]) && !node.hidden {
                        results.push(&node.cmd);
                    }
                }
                Some(results)
            }
            else {
                results = nodes.iter().map(|n| n.cmd.as_str()).collect();
                if let Some(cmd) = parent {
                    if **cmd == self.cmd {
                        Some(results)
                    }
                    else {
                        None
                    }
                }
                else {
                    Some(results)
                }
            }
        }
        else {
            None
        }
    }

    pub fn args(&self) -> &Option<Vec<(String, bool)>> {
        &self.args
    }

    pub fn print_help(&self, level: u8) {
        if self.hidden {
            return;
        }
        if level > 0 {
            for _ in 0..level {
                print!("  ");
            }
            print!("{}:\t{}", self.cmd, self.help);
            if let Some(ref args) = self.args {
                let mut required = Vec::with_capacity(args.len());
                let mut optional = Vec::with_capacity(args.len());

                for &(ref arg, ref req) in args.iter() {
                    if *req {
                        required.push(arg);
                    }
                    else {
                        optional.push(arg);
                    }
                }

                if required.len() > 0 || optional.len() > 0 {
                    print!(", Format: {} ", self.cmd);
                    for arg in required {
                        print!("<{}> ", arg);
                    }
                    for arg in optional {
                        print!("[{}] ", arg);
                    }
                    if self.sub_nodes.is_some() {
                        print!("[enter]");
                    }
                }
            }
            println!("");
        }
        else {
            println!("{}: {}", self.cmd, self.help);
            println!("-------------------");
            println!("Commands:");
        }
        if let Some(ref nodes) = self.sub_nodes {
            for node in nodes {
                node.print_help(level + 1)
            };
        }
    }
}

#[macro_export]
macro_rules! shell_command_node {
    (
        cmd: $name:ident,
        txt_help: $help:expr
    ) => {
        $crate::commands::Node::new(stringify!($name), None, $help, None);
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        nodes: [ $( $node:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), None, $help, None);
            $(
                this_node.add_node($node);
            )*
            this_node
        }
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        args: [ $( $arg:ident => $required:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), None, $help, None);
            $(
                this_node.add_arg(stringify!($arg), $required);
            )*
            this_node
        }
    };
    (
        cmd: $name:ident,
        conditional: $cond:expr,
        txt_help: $help:expr,
        nodes: [ $( $node:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), Some($cond), $help, None);
            $(
                this_node.add_node($node);
            )*
            this_node
        }
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        callback: $callback:expr,
        args: [ $( $arg:ident => $required:expr ),* ],
        nodes: [ $( $node:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), None, $help, Some(Box::new($callback)));
            $(
                this_node.add_node($node);
            )*
            $(
                this_node.add_arg(stringify!($arg), $required);
            )*
            this_node
        }
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        callback: $callback:expr
    ) => {
        $crate::commands::Node::new(stringify!($name), None, $help, Some(Box::new($callback)));
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        callback: $callback:expr,
        nodes: [ $( $node:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), None, $help, Some(Box::new($callback)));
            $(
                this_node.add_node($node);
            )*
            this_node
        }
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        callback: $callback:expr,
        args: [ $( $arg:ident => $required:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), None, $help, Some(Box::new($callback)));
            $(
                this_node.add_arg(stringify!($arg), $required);
            )*
            this_node
        }
    };
    (
        cmd: $name:ident,
        txt_help: $help:expr,
        callback: $callback:expr,
        args: [ $( $arg:ident => $required:expr ),* ],
        nodes: [ $( $node:expr ),* ]
    ) => {
        {
            let mut this_node = $crate::commands::Node::new(stringify!($name), None, $help, Some(Box::new($callback)));
            $(
                this_node.add_node($node);
            )*
            $(
                this_node.add_arg(stringify!($arg), $required);
            )*
            this_node
        }
    };
}


pub struct CommandTree<T>
    where T: Debug
{
    name: String,
    version: String,
    root: Node<T>,
    context: Arc<Mutex<Option<T>>>
}

impl <T>CommandTree<T>
    where T: Debug
{
    pub fn new(name: &str, version: &str, root: Node<T>, context: Option<T>) -> CommandTree<T> {
        CommandTree {
            name: name.to_owned(),
            version: version.to_owned(),
            root: root,
            context: Arc::new(Mutex::new((context)))
        }
    }

    pub fn get_help(&self) {
        self.root.print_help(0);
    }

    pub fn get_suggestions<'a>(&'a self, line: &str, o_sub_node: Option<&'a Node<T>>) -> Option<Vec<&str>> {
        let mut levels: Vec<&str> = line.trim_left().split_whitespace().collect();
        if line.ends_with(char::is_whitespace) {
            levels.push("");
        }
        debug!("Levels: {:?}", levels);
        let mut node = &self.root;
        if let Some(sub_node) = o_sub_node {
            node = sub_node;
        }

        let mut i = 0;
        let mut nr_required = 0;
        while i != levels.len() {
            debug!("i: {}, looking for: {}", i, levels[i]);
            node = match node.find(levels[i]) {
                Some(current_node) => {
                    if let Some(ref args) = current_node.args {
                        let required: Vec<&str> = args.iter().filter_map(|a| {
                            if a.1 {
                                Some(a.0.as_str())
                            }
                            else {
                                None
                            }
                        }).collect();

                        nr_required = required.len();
                        for (x, _) in required.iter().enumerate() {
                            if let None = levels.get(i + 1) {
                                println!("'{}' missing fields: {:?}", current_node.cmd, required[x..].to_vec());
                                return None;
                            }
                            i += 1;
                        }
                    }
                    current_node
                },
                None => {
                    break;
                }
            };
            info!("new node: {}", node.cmd);
            i += 1;
        }

        node.get_suggestions(levels, i, nr_required)
    }

    fn gen_prompt(&self, o_my_context: &Option<Vec<(&Node<T>, String)>>) -> String {
        if let Some(ref my_context) = *o_my_context {
            let mut temp_prompt = format!("{}", self.name);
            for entry in my_context.iter() {
                temp_prompt = format!("{}/{}", temp_prompt, entry.1);
            }
            format!("\x1b[1;32m{}>>\x1b[0m ", temp_prompt)
        }
        else {
            format!("\x1b[1;32m{}>>\x1b[0m ", self.name)
        }
    }


    pub fn run(&mut self)  {
        println!("Welcome to {} v{}", self.name, self.version);
        // let config = Config::builder()
        //     .history_ignore_space(true)
        //     .completion_type(CompletionType::List)
        //     .build();

        let c = TxCompleter::new(&self, None);
        let mut rl: Editor<TxCompleter<T>> = Editor::new().history_ignore_space(true);
        rl.set_completer(Some(c));

        let home_dir = env::home_dir()
            .and_then(|dir| {
                Some(dir.to_str().unwrap_or("/tmp").to_owned())
            })
            .unwrap();
        let history_file = format!("{}/{}", home_dir, ".txcli_history");

        if rl.load_history(&history_file).is_err() {
            println!("No previous history.");
        }

        let mut prompt = format!("\x1b[1;32m{}>>\x1b[0m ", self.name);
        let mut ops = 0u64;
        let mut o_my_context: Option<Vec<(&Node<T>, String)>> = None;
        let mut history: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = match rl.readline(&prompt) {
                Ok(val) => val,
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => continue,
                Err(e) => {
                    println!("Readline error: {}", e);
                    break;
                }
            };

            if !line.starts_with("profile") || line.contains("login") {
                rl.add_history_entry(line.as_str());
            }

            if line.contains('>') {
                let v_line: Vec<String> = line.split('>').map(|e| e.to_owned()).collect();
                if v_line.len() != 2 {
                    println!("Wrong cmd format in line: {}", line);
                    continue;
                }
                line = v_line[0].trim().to_owned();
                let output = v_line[1].trim();
                if output.contains(char::is_whitespace)  {
                    println!("Wrong cmd format in output: {}", output);
                    continue;
                }
                history.insert("tx_output_file".to_owned(), output.to_owned());
            }
            else if line.contains('|') {
                let v_line: Vec<String> = line.split('|').map(|e| e.to_owned()).collect();
                if v_line.len() != 2 {
                    println!("Wrong cmd format in line: {}", line);
                    continue;
                }
                line = v_line[0].trim().to_owned();
                let output = v_line[1].trim();
                if output.contains(char::is_whitespace)  {
                    println!("Wrong cmd format in output: {}", output);
                    continue;
                }
                history.insert("tx_modifier".to_owned(), output.to_owned());
            }
            else {
                history.remove("tx_modifier");
                history.remove("tx_output_file");
            }

            if line.contains('<') {
                let v_line: Vec<String> = line.split('<').map(|e| e.to_owned()).collect();
                if v_line.len() != 2 {
                    println!("Wrong cmd format in line: {}", line);
                    continue;
                }
                line = format!("{} <", v_line[0].trim());
                let input = v_line[1].trim();
                if input.contains(char::is_whitespace)  {
                    println!("Wrong cmd format in input: {}", input);
                    continue;
                }
                history.insert("tx_input_file".to_owned(), input.to_owned());
            }
            else {
                history.remove("tx_input_file");
            }

            match line.trim() {
                "exit" | "quit" => {
                    if let Err(e) = rl.save_history(&history_file) {
                        println!("Could not save history. Err: {}", e);
                    }
                    ::std::process::exit(0);
                },
                "top" => {
                    rl.set_completer(Some(TxCompleter::new(&self, None)));
                    o_my_context = None;
                    history.clear();
                    prompt = self.gen_prompt(&o_my_context);
                    continue;
                },
                "up" => {
                    if let Some(ref mut my_context) = o_my_context {
                        my_context.pop();
                        if let Some(&(context, _)) = my_context.last() {
                            rl.set_completer(Some(TxCompleter::new(&self, Some(context))));
                        }
                        else {
                            rl.set_completer(Some(TxCompleter::new(&self, None)));
                        }
                    }
                    else {
                        rl.set_completer(Some(TxCompleter::new(&self, None)));
                        history.clear();
                    }
                    prompt = self.gen_prompt(&o_my_context);
                    continue;
                },
                _ => ()
            };

            if line.starts_with("help") {
                self.get_help();
                continue;
            }

            let mut levels = Vec::new();
            for (i, entry) in line.trim().split('"').enumerate() {
                if i % 2 == 0 {
                    if !entry.is_empty() {
                        for token in entry.trim().split_whitespace() {
                            if !token.is_empty() {
                                levels.push(token);
                            }
                        }
                    }
                }
                else {
                    if !entry.is_empty() {
                        levels.push(entry);
                    }
                }
            }

            let mut node = &self.root;
            if let Some(ref my_context) = o_my_context {
                if let Some(&(context, _)) = my_context.last() {
                    node = context;
                }
            }

            let mut i = 0;
            let mut error = false;
            while i != levels.len() {
                debug!("i: {}, looking for: {}", i, levels[i]);
                node = match node.find(levels[i]) {
                    Some(current_node) => {
                        if let Some(ref callback) = current_node.callback {
                            let mut my_args: HashMap<String, &str> = HashMap::new();
                            if let Some(ref args) = current_node.args {
                                let required: Vec<&str> = args.iter().filter_map(|a| {
                                    if a.1 {
                                        Some(a.0.as_str())
                                    }
                                    else {
                                        None
                                    }
                                }).collect();
                                for (x, arg) in required.iter().enumerate() {
                                    if let Some(val) = levels.get(i + 1) {
                                        if *val == "?" {
                                            current_node.print_help(1);
                                            error = true;
                                            break;
                                        }
                                        my_args.insert((*arg).to_owned(), val);
                                    }
                                    else {
                                        println!("'{}' missing fields: {:?}", current_node.cmd, required[x..].to_vec());
                                        error = true;
                                        break;
                                    }
                                    i += 1;
                                }
                                if error {
                                    break;
                                }
                            }

                            debug!("Current: {:?}, args: {:?}, levels[i]: {:?}", current_node.cmd, my_args, levels.get(i + 1));
                            let mut context = match self.context.lock() {
                                Ok(val) => val,
                                Err(e) => {
                                    println!("Could not lock context. Err: {}", e);
                                    continue;
                                }
                            };
                            match callback(my_args, &mut context, &history) {
                                Ok(Some(str_context)) => {
                                    if str_context == "up" {
                                        if let Some(ref mut my_context) = o_my_context {
                                            my_context.pop();
                                            if let Some(&(context, _)) = my_context.last() {
                                                rl.set_completer(Some(TxCompleter::new(&self, Some(context))));
                                            }
                                        }
                                        else {
                                            rl.set_completer(Some(TxCompleter::new(&self, None)));
                                            history.clear();
                                        }
                                        prompt = self.gen_prompt(&o_my_context);
                                        break;
                                    };
                                    let cmd_context: Vec<&str> = str_context.split(':').collect();
                                    if cmd_context.len() < 2 {
                                        panic!("Context switcher should return string 'object:name'");
                                    }
                                    let cmd = cmd_context[0];
                                    let new_context = cmd_context[1];
                                    debug!("new_context: {}", new_context);
                                    if levels.get(i + 1).is_some() {
                                        history.insert(current_node.cmd.to_owned(), new_context.to_owned());
                                    }
                                    else {
                                        rl.set_completer(Some(TxCompleter::new(&self, Some(current_node))));
                                        history.insert(cmd.to_owned(), new_context.to_owned());
                                        if let Some(ref mut my_context) = o_my_context {
                                            my_context.push((current_node, new_context.to_owned()));
                                        }
                                        else {
                                            o_my_context = Some(vec![(current_node, new_context.to_owned())]);
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("Error: {}", e);
                                },
                                _ => ()
                            }
                        }
                        current_node
                    },
                    None => {
                        if !levels[i].starts_with("?") {
                            println!("Error: command not found");
                        }
                        break;
                    }
                };
                i += 1;
            }
            if error {
                continue;
            }

            if ops % 5 == 0 {
                if let Err(e) = rl.save_history(&history_file) {
                    println!("Could not save history. Err: {}", e);
                }
            }
            ops += 1;
            prompt = self.gen_prompt(&o_my_context);
        }
    }
}

pub fn exit_cli<T>(_args: HashMap<String, &str>, _: &mut Option<T>, _: &HashMap<String, String>) -> CommandResult<Option<String>> {
    ::std::process::exit(0);
}

#[macro_export]
macro_rules! shell_command_tree {
    (
        $name:ident,
        $help: expr,
        $version: expr,
        $context: expr,
        [ $( $node:expr ),* ]
    ) => {
        {
            let mut root_node = $crate::commands::Node::new(stringify!($name), None, $help, None);
            $(
                root_node.add_node($node);
            )*
            root_node.add_node(shell_command_node!{
                cmd: exit,
                txt_help: "Exit Shell",
                callback: $crate::commands::exit_cli
            });
            root_node.add_node(shell_command_node!{
                cmd: quit,
                txt_help: "Exit Shell",
                callback: $crate::commands::exit_cli
            });
            CommandTree::new(stringify!($name), $version, root_node, Some($context))
        }
    };
    (
        $name:ident,
        $help: expr,
        $version: expr,
        [ $( $node:expr ),* ]
    ) => {
        {
            let mut root_node = $crate::commands::Node::new(stringify!($name), $help, None);
            $(
                root_node.add_node($node);
            )*
            CommandTree::new(stringify!($name), $version, root_node, None)
        }
    };
}
