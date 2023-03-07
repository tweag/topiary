/// GraphViz visualisation for our SyntaxTree representation
/// Named syntax nodes are elliptical; anonymous are rectangular
use std::{fmt, io};

use crate::{tree_sitter::SyntaxNode, FormatterResult};

pub enum Colour {
    RGBA(u8, u8, u8, Option<u8>),
    HSV(f32, f32, f32),
    Named(NamedColour),
}

pub enum NamedColour {
    Black,
    Red,
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RGBA(r, g, b, Some(a)) => write!(f, "#{r:02x}{g:02x}{b:02x}{a:02x}"),
            Self::RGBA(r, g, b, None) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
            Self::HSV(h, s, v) => write!(f, "{h:.3} {s:.3} {v:.3}"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

impl fmt::Display for NamedColour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Black => write!(f, "black"),
            Self::Red => write!(f, "red"),
        }
    }
}

pub enum Shape {
    Box,
    Ellipse,
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Box => write!(f, "box"),
            Self::Ellipse => write!(f, "ellipse"),
        }
    }
}

pub trait GraphViz {
    fn uid(&self) -> String;
    fn label(&self) -> Option<String>;
    fn shape(&self) -> Option<Shape>;
    fn colour(&self) -> Option<Colour>;
    fn children(&self) -> Vec<&Self>;

    fn as_dot(&self, f: &mut dyn io::Write) -> FormatterResult<()> {
        let has_attrs = self.label().is_some() || self.shape().is_some() || self.colour().is_some();

        write!(f, "{}", self.uid())?;
        if has_attrs {
            write!(f, " [")?;

            if let Some(label) = self.label() {
                write!(f, "label=\"{label}\" ")?;
            }

            if let Some(shape) = self.shape() {
                write!(f, "shape={shape} ")?;
            }

            if let Some(colour) = self.colour() {
                write!(f, "color={colour} fontcolor={colour} ")?;
            }

            write!(f, "]")?;
        }
        writeln!(f, ";")?;

        for child in self.children() {
            writeln!(f, "{} -- {};", self.uid(), child.uid())?;
            child.as_dot(f)?;
        }

        Ok(())
    }
}

impl GraphViz for SyntaxNode {
    fn uid(&self) -> String {
        self.id.to_string()
    }

    fn label(&self) -> Option<String> {
        Some(self.kind.escape_default().collect())
    }

    fn shape(&self) -> Option<Shape> {
        match self.is_named {
            true => Some(Shape::Ellipse),
            false => Some(Shape::Box),
        }
    }

    fn colour(&self) -> Option<Colour> {
        Some(Colour::Named(NamedColour::Red))
    }

    fn children(&self) -> Vec<&Self> {
        self.children.iter().collect()
    }
}

pub fn write<T: GraphViz>(output: &mut dyn io::Write, root: &T) -> FormatterResult<()> {
    writeln!(output, "graph {{")?;
    root.as_dot(output)?;
    writeln!(output, "}}")?;

    Ok(())
}
