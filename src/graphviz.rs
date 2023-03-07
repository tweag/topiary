/// GraphViz visualisation for our SyntaxTree representation
/// Named syntax nodes are elliptical; anonymous are rectangular
use std::{fmt, io};

use crate::{tree_sitter::SyntaxNode, FormatterResult};

impl fmt::Display for SyntaxNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let shape = match self.is_named {
            true => "ellipse",
            false => "box",
        };

        writeln!(
            f,
            "  {} [label=\"{}\", shape={shape}];",
            self.id,
            self.kind.escape_default()
        )?;

        for child in &self.children {
            writeln!(f, "  {} -- {};", self.id, child.id)?;
            write!(f, "{child}")?;
        }

        Ok(())
    }
}

pub fn write(output: &mut dyn io::Write, root: &SyntaxNode) -> FormatterResult<()> {
    writeln!(output, "graph {{")?;
    write!(output, "{root}")?;
    writeln!(output, "}}")?;

    Ok(())
}
