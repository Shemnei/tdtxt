use std::fmt;

use crate::parse::{Parse, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
	Open,
	Done,
}

impl Default for State {
	fn default() -> Self {
		Self::Open
	}
}

impl fmt::Display for State {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Done => f.write_str("x"),
			Self::Open => Ok(()),
		}
	}
}

crate::parse_error!(ParseStateError : "state");

impl Parse for State {
	type Error = ParseStateError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		if parser.expect_u8(b'x').is_some() {
			Ok(Self::Done)
		} else {
			Err(ParseStateError::default())
		}
	}
}

crate::impl_fromstr!(State);
