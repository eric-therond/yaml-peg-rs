//! Dumper components.
use crate::{repr::Repr, *};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Newline symbol in common platforms.
///
/// There is only two symbols in the option,
/// "\r\n" in Windows platform, otherwise uses "\n".
///
/// Please be aware that your storage can be used the symbol of Windows.
pub const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

#[derive(Eq, PartialEq)]
enum Root {
    Scalar,
    Map,
    Array,
}

/// Dumper for nodes.
pub struct Dumper<'a, R: Repr> {
    node: &'a Node<R>,
    root: Root,
    level: usize,
}

impl<'a, R: Repr> Dumper<'a, R> {
    /// Create the dumper.
    pub fn new(node: &'a Node<R>) -> Self {
        Self {
            node,
            root: Root::Scalar,
            level: 0,
        }
    }

    fn part(node: &'a Node<R>, root: Root, level: usize) -> String {
        Self { node, root, level }.dump()
    }

    /// Dump into string.
    pub fn dump(&self) -> String {
        let mut doc = String::new();
        let tag = self.node.tag();
        if !tag.is_empty() && !tag.starts_with(parser::tag_prefix!()) {
            doc += &if tag.starts_with(parser::tag_prefix!()) {
                format!("!!{} ", tag)
            } else if parser::Parser::new(tag.as_bytes()).identifier().is_ok() {
                format!("!{} ", tag)
            } else {
                format!("!<{}> ", tag)
            };
        }
        let ind = "  ".repeat(self.level);
        doc += &match &self.node.yaml() {
            Yaml::Null => "null".to_string(),
            Yaml::Bool(b) => b.to_string(),
            Yaml::Int(n) | Yaml::Float(n) => n.clone(),
            Yaml::Str(s) => {
                if s.contains(NL) {
                    let s = s
                        .split(NL)
                        .map(|s| {
                            if s.is_empty() {
                                String::new()
                            } else {
                                ind.to_string() + s.trim_end()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(NL);
                    format!("|{}{}{}", NL, ind, s.trim())
                } else if parser::Parser::new(s.as_bytes())
                    .string_plain(0, false)
                    .is_err()
                {
                    format!("\"{}\"", s)
                } else {
                    s.clone()
                }
            }
            Yaml::Seq(a) => {
                let mut doc = NL.to_string();
                for (i, node) in a.iter().enumerate() {
                    if i != 0 || self.level != 0 {
                        doc += &ind;
                    }
                    let s = Self::part(node, Root::Array, self.level + 1);
                    doc += &format!("- {}{}", s, NL);
                }
                doc.truncate(doc.len() - NL.len());
                doc
            }
            Yaml::Map(m) => {
                let mut doc = match self.root {
                    Root::Map => NL.to_string(),
                    _ => String::new(),
                };
                for (i, (k, v)) in m.iter().enumerate() {
                    if i != 0 || self.root == Root::Map {
                        doc += &ind;
                    }
                    let s = Self::part(k, Root::Map, self.level + 1);
                    doc += &if let Yaml::Map(_) | Yaml::Seq(_) = k.yaml() {
                        let pre_ind = "  ".repeat(self.level + 1);
                        format!("?{}{}{}{}{}", pre_ind, NL, s, NL, ind)
                    } else {
                        s
                    };
                    doc += ":";
                    doc += &match v.yaml() {
                        Yaml::Map(_) => Self::part(v, Root::Map, self.level + 1),
                        Yaml::Seq(_) if self.root == Root::Array && i == 0 => {
                            Self::part(v, Root::Map, self.level)
                        }
                        Yaml::Seq(_) => Self::part(v, Root::Map, self.level + 1),
                        _ => format!(" {}", Self::part(v, Root::Map, self.level + 1)),
                    };
                    doc += NL;
                }
                doc.truncate(doc.len() - NL.len());
                doc
            }
        };
        doc
    }
}

/// Dump the YAML data in to block format.
///
/// Dumper will use plain string when the string is none-wrapped,
/// otherwise it use literal string and trim the last white spaces.
///
/// ```
/// use yaml_peg::{dump, node, dumper::NL};
///
/// let doc = dump(&[
///     node!({
///         "a" => "b",
///         "c" => "d",
///     }),
/// ]);
/// let ans = "\
/// a: b
/// c: d
/// ";
/// assert_eq!(doc, ans.replace('\n', NL));
/// ```
///
/// When calling [`parse`] function then [`dump`] the string, the string can be reformatted.
pub fn dump<R: Repr>(nodes: &[Node<R>]) -> String {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let doc = Dumper::new(node).dump() + NL;
            match i {
                0 => doc,
                _ => format!("---{}{}", NL, doc.trim_start()),
            }
        })
        .collect()
}
