use std::fmt::Debug;
use rustyline::completion::Completer;
use rustyline::Result;
use super::CommandTree;
use super::Node;

pub struct TxCompleter<'a, T>
    where T: 'a + Debug
{
    tree: &'a CommandTree<T>,
    sub_node: Option<&'a Node<T>>
}

impl <'a, T>TxCompleter<'a, T>
    where T: Debug
{
    pub fn new(tree: &'a CommandTree<T>, sub_node: Option<&'a Node<T>>) -> TxCompleter<'a, T> {
        TxCompleter {
            tree: tree,
            sub_node: sub_node
        }
    }
}

impl <'a, T>Completer for TxCompleter<'a, T>
    where T: Debug
{
    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<String>)> {
        debug!("Completion on line: {}, pos: {}", line, pos);
        let suggestions: Vec<String> = self.tree.get_suggestions(line, self.sub_node)
            .and_then(|mut entries| {
                Some(entries.drain(..).map(|entry| entry.to_owned()).collect())
            })
            .unwrap_or(Vec::new());

        let insert_pos = match line.rfind(char::is_whitespace) {
            Some(val) => val + 1,
            None => 0
        };

        Ok((insert_pos, suggestions))
    }

}
