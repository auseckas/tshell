use std::fmt::Debug;
use rustyline::line_buffer::LineBuffer;
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

        Ok((pos, suggestions))
    }

    fn update(&self, line_buffer: &mut LineBuffer, _start: usize, elected: &str) {
        let line = line_buffer.as_str().to_owned();

        let words: Vec<&str> = line.split(char::is_whitespace).collect();

        if let Some(last_word) = words.last() {
            if elected.starts_with(last_word) {
                let insert_pos = match line.rfind(char::is_whitespace) {
                    Some(val) => val + 1,
                    None => 0
                };
                debug!("Insert pos: {}, buf len: {}", insert_pos, line_buffer.len());
                let end = line_buffer.pos();
                line_buffer.replace(insert_pos, end, elected);
            }
        }
        line_buffer.move_end();
    }

}
