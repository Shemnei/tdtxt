use std::fmt;

use crate::parse::{Parse, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
	Done,
}

impl fmt::Display for State {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Done => f.write_str("x"),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateParseError;

impl fmt::Display for StateParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("failed to parse state")
	}
}

impl std::error::Error for StateParseError {}

impl Parse for State {
	type Error = StateParseError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		if parser.expect_slice(b"x ").is_some() {
			Ok(Self::Done)
		} else {
			Err(StateParseError)
		}
	}
}
