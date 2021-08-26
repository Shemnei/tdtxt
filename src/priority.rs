use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;

use crate::parse::{Parse, Parser};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
	A = 0,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
	I,
	J,
	K,
	L,
	M,
	N,
	O,
	P,
	Q,
	R,
	S,
	T,
	U,
	V,
	W,
	X,
	Y,
	Z,
}

impl PartialOrd<Self> for Priority {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(std::cmp::Ord::cmp(self, other))
	}
}

impl Ord for Priority {
	fn cmp(&self, other: &Self) -> Ordering {
		match std::cmp::Ord::cmp(&(*self as u8), &(*other as u8)) {
			Ordering::Less => Ordering::Greater,
			Ordering::Greater => Ordering::Less,
			Ordering::Equal => Ordering::Equal,
		}
	}
}

impl fmt::Display for Priority {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::A => f.write_str("(A)"),
			Self::B => f.write_str("(B)"),
			Self::C => f.write_str("(C)"),
			Self::D => f.write_str("(D)"),
			Self::E => f.write_str("(E)"),
			Self::F => f.write_str("(F)"),
			Self::G => f.write_str("(G)"),
			Self::H => f.write_str("(H)"),
			Self::I => f.write_str("(I)"),
			Self::J => f.write_str("(J)"),
			Self::K => f.write_str("(K)"),
			Self::L => f.write_str("(L)"),
			Self::M => f.write_str("(M)"),
			Self::N => f.write_str("(N)"),
			Self::O => f.write_str("(O)"),
			Self::P => f.write_str("(P)"),
			Self::Q => f.write_str("(Q)"),
			Self::R => f.write_str("(R)"),
			Self::S => f.write_str("(S)"),
			Self::T => f.write_str("(T)"),
			Self::U => f.write_str("(U)"),
			Self::V => f.write_str("(V)"),
			Self::W => f.write_str("(W)"),
			Self::X => f.write_str("(X)"),
			Self::Y => f.write_str("(Y)"),
			Self::Z => f.write_str("(Z)"),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidPriorityError;

impl fmt::Display for InvalidPriorityError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("invalid priority")
	}
}

impl std::error::Error for InvalidPriorityError {}

impl TryFrom<char> for Priority {
	type Error = InvalidPriorityError;

	fn try_from(value: char) -> Result<Self, Self::Error> {
		match value {
			'A' => Ok(Self::A),
			'B' => Ok(Self::B),
			'C' => Ok(Self::C),
			'D' => Ok(Self::D),
			'E' => Ok(Self::E),
			'F' => Ok(Self::F),
			'G' => Ok(Self::G),
			'H' => Ok(Self::H),
			'I' => Ok(Self::I),
			'J' => Ok(Self::J),
			'K' => Ok(Self::K),
			'L' => Ok(Self::L),
			'M' => Ok(Self::M),
			'N' => Ok(Self::N),
			'O' => Ok(Self::O),
			'P' => Ok(Self::P),
			'Q' => Ok(Self::Q),
			'R' => Ok(Self::R),
			'S' => Ok(Self::S),
			'T' => Ok(Self::T),
			'U' => Ok(Self::U),
			'V' => Ok(Self::V),
			'W' => Ok(Self::W),
			'X' => Ok(Self::X),
			'Y' => Ok(Self::Y),
			'Z' => Ok(Self::Z),
			_ => Err(InvalidPriorityError),
		}
	}
}

impl TryFrom<u8> for Priority {
	type Error = InvalidPriorityError;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Self::try_from(value as char)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PriorityParseError;

impl fmt::Display for PriorityParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("failed to parse priority")
	}
}

impl std::error::Error for PriorityParseError {}

impl Parse for Priority {
	type Error = PriorityParseError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let _ = parser.expect_u8(b'(').ok_or(PriorityParseError)?;
		let priority = parser.parse_alpha_upper().ok_or(PriorityParseError)?;
		let priority =
			Self::try_from(priority).map_err(|_| PriorityParseError)?;
		let _ = parser.expect_slice(b") ").ok_or(PriorityParseError)?;
		Ok(priority)
	}
}
