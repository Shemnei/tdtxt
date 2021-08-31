use std::cmp::Ordering;
use std::convert::TryFrom;

use crate::parse::{Parse, Parser};

macro_rules! priorities {
	(
		$(
			$( #[doc = $doc:literal] )*
			$name:ident : $char:literal $( = $idx:literal )? ,
		)+
	) => {
		/// Represents the priority a task can have.
		///
		/// # Notes
		///
		/// [`Priority::A`] has the lowest integer code but is considered to be
		/// the highest priority there is. [`Priority::Z`] is the lowest
		/// priority.
		#[repr(u8)]
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub enum Priority {
			$(
				$( #[doc = $doc] )*
				$name $( = $idx )? ,
			)+
		}

		impl ::std::fmt::Display for Priority {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				match self {
					$( Self::$name => f.write_str(concat!("(", stringify!($name) ,")")) , )+
				}
			}
		}

		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub struct InvalidPriorityError;

		impl ::std::fmt::Display for InvalidPriorityError {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				f.write_str("invalid priority")
			}
		}

		impl ::std::error::Error for InvalidPriorityError {}

		impl ::std::convert::TryFrom<char> for Priority {
			type Error = InvalidPriorityError;

			fn try_from(value: char) -> ::std::result::Result<Self, Self::Error> {
				match value {
					$( $char => Ok(Self::$name) , )+
					_ => Err(InvalidPriorityError),
				}
			}
		}
	};
}

priorities! {
	/// Priority `A`. Highest priority.
	A : 'A' = 0,
	/// Priority `B`.
	B : 'B',
	/// Priority `C`.
	C : 'C',
	/// Priority `D`.
	D : 'D',
	/// Priority `E`.
	E : 'E',
	/// Priority `F`.
	F : 'F',
	/// Priority `G`.
	G : 'G',
	/// Priority `H`.
	H : 'H',
	/// Priority `I`.
	I : 'I',
	/// Priority `J`.
	J : 'J',
	/// Priority `K`.
	K : 'K',
	/// Priority `L`.
	L : 'L',
	/// Priority `M`.
	M : 'M',
	/// Priority `N`.
	N : 'N',
	/// Priority `O`.
	O : 'O',
	/// Priority `P`.
	P : 'P',
	/// Priority `Q`.
	Q : 'Q',
	/// Priority `R`.
	R : 'R',
	/// Priority `S`.
	S : 'S',
	/// Priority `T`.
	T : 'T',
	/// Priority `U`.
	U : 'U',
	/// Priority `V`.
	V : 'V',
	/// Priority `W`.
	W : 'W',
	/// Priority `X`.
	X : 'X',
	/// Priority `Y`.
	Y : 'Y',
	/// Priority `Z`. Lowest priority.
	Z : 'Z',
}

impl PartialOrd<Self> for Priority {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(std::cmp::Ord::cmp(self, other))
	}
}

impl Ord for Priority {
	fn cmp(&self, other: &Self) -> Ordering {
		// Switched (other with self) so that `0` is the highest priority
		std::cmp::Ord::cmp(&(*other as u8), &(*self as u8))
	}
}

crate::parse_error!(ParsePriorityError: "priority");

impl Parse for Priority {
	type Error = ParsePriorityError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let _ =
			parser.expect_u8(b'(').ok_or_else(ParsePriorityError::default)?;
		let priority = parser
			.parse_alpha_upper()
			.ok_or_else(ParsePriorityError::default)?;
		let priority = Self::try_from(priority)
			.map_err(|_| ParsePriorityError::default())?;
		let _ =
			parser.expect_u8(b')').ok_or_else(ParsePriorityError::default)?;
		Ok(priority)
	}
}

crate::impl_fromstr!(Priority);
