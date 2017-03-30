extern crate rustyline;
#[macro_use]
extern crate log;

#[macro_use]
pub mod commands;
pub mod completion;
pub use commands::{CommandTree, CommandResult, Node};
