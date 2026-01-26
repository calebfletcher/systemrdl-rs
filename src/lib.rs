// Licensed under the Apache-2.0 license.

//! General-purpose parser for systemrdl files.

pub mod ast;
mod bits;
mod elaborator;
mod file_source;
mod lexer;
mod parser;
mod string_arena;
mod token;
mod token_iter;

pub use bits::Bits;
pub use elaborator::elaborate;
pub use file_source::{FileSource, FsFileSource};
pub use parser::parse;
pub use token::*;
