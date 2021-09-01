use std::fmt;

use crate::parse::{Parse, Parser};

/// Represents the state of [`Task`](`crate::Task`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
	/// The task is still open e.g. not done (no representation).
	Open,

	/// The task is done (representation: `x`).
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
