use crate::span::BytePos;

pub trait Parse: Sized {
	type Error: std::error::Error;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error>;

	fn parse_opt(parser: &mut Parser<'_>) -> Option<Self> {
		let parser_pre = *parser;

		match Self::parse(parser) {
			Err(_) => {
				*parser = parser_pre;
				None
			}
			Ok(x) => Some(x),
		}
	}
}

/// Generates a basic generic error type for use in parsing.
///
/// An optional message can be associated with the generated error type when
/// instantiated with `Self::with_msg`.
#[macro_export]
macro_rules! parse_error {
	( $name:ident : $ty:literal ) => {
		#[doc = concat!("A generic error which can occur during parsing of a ", $ty, ".")]
		#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
		pub struct $name {
			/// Optional attached message/description.
			///
			/// This can be used for a more detailed description of what went
			/// wrong.
			msg: ::std::option::Option<::std::borrow::Cow<'static, str>>,
		}

		impl $name {
			/// Creates a new instance and associates `msg` with it.
			///
			/// # Examples
			///
			/// ```rust,ignore
			#[doc = concat!(" let error: ", stringify!($name), " = ", stringify!($name), r#"::with_msg("detailed message");"#)]
			/// ```
			fn with_msg<M: ::std::convert::Into<::std::borrow::Cow<'static, str>>>(
				msg: M,
			) -> Self {
				Self { msg: ::std::option::Option::Some(msg.into()) }
			}
		}

		impl ::std::fmt::Display for $name {
			fn fmt(
				&self,
				f: &mut ::std::fmt::Formatter<'_>,
			) -> ::std::fmt::Result {
				if let ::std::option::Option::Some(msg) = &self.msg {
					write!(f, concat!("failed to parse ", $ty, ": {}"), msg)
				} else {
					f.write_str(concat!("failed to parse ", $ty))
				}
			}
		}

		impl ::std::error::Error for $name {}
	};
}

/// Implements [`std::str::FromStr`] for a type which implements
/// [`Parse`](`crate::parse::Parse`).
///
/// # Notes
///
/// For this to work, [`Parse::Error`](`crate::parse::Parse::Error`) must
/// implement a function with the following signature `fn with_msg(&str) ->
/// Self`. If the macro [`parse_error`] is used, this function will be
/// implemented automatically.
#[macro_export]
macro_rules! impl_fromstr {
	( $ty:ty ) => {
		impl ::std::str::FromStr for $ty {
			type Err = <Self as Parse>::Error;

			fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
				let mut parser = Parser::new(s.as_bytes());

				let tmp = <$ty>::parse(&mut parser)?;

				if parser.is_eof() {
					Ok(tmp)
				} else {
					Err(<Self as Parse>::Error::with_msg(
						"more tokens in input",
					))
				}
			}
		}
	};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parser<'a> {
	cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
	pub const fn new(bytes: &'a [u8]) -> Self {
		Self { cursor: Cursor::new(bytes) }
	}

	pub const fn is_eof(&self) -> bool {
		self.cursor.is_eof()
	}

	pub fn parse_u8(&mut self) -> Option<u8> {
		self.cursor.consume()
	}

	pub fn parse_alpha(&mut self) -> Option<char> {
		match self.parse_alpha_lower() {
			None => self.parse_alpha_upper(),
			x => x,
		}
	}

	pub fn parse_alpha_lower(&mut self) -> Option<char> {
		match self.cursor.first() {
			Some(x @ b'a'..=b'z') => {
				self.cursor.advance(1);
				Some(x as char)
			}
			_ => None,
		}
	}

	pub fn parse_alpha_upper(&mut self) -> Option<char> {
		match self.cursor.first() {
			Some(x @ b'A'..=b'Z') => {
				self.cursor.advance(1);
				Some(x as char)
			}
			_ => None,
		}
	}

	pub fn parse_digit(&mut self) -> Option<u8> {
		match self.cursor.first() {
			Some(x @ b'0'..=b'9') => {
				self.cursor.advance(1);
				Some(x - b'0')
			}
			_ => None,
		}
	}

	pub fn parse_until(&mut self, terminator: u8) -> Option<&[u8]> {
		if self.cursor.is_eof() {
			None
		} else {
			let start = self.cursor.index();
			self.cursor.consume_while(|b| b != terminator);
			Some(&self.cursor.bytes[start..self.cursor.index])
		}
	}

	pub fn expect_u8(&mut self, expect: u8) -> Option<u8> {
		if matches!(self.cursor.first(), Some(x) if x == expect) {
			self.cursor.advance(1);
			Some(expect)
		} else {
			None
		}
	}

	pub fn expect_whitespace(&mut self) -> Option<()> {
		if matches!(self.cursor.first(), Some(x) if x.is_ascii_whitespace()) {
			self.cursor.advance(1);
			Some(())
		} else {
			None
		}
	}

	pub fn expect_slice<S>(&mut self, expect: S) -> Option<&[u8]>
	where
		S: AsRef<[u8]>,
	{
		let expect = expect.as_ref();
		let len = expect.len();
		let index_end = self.cursor.index + if len > 0 { len - 1 } else { 0 };

		if self.cursor.in_bounds(index_end) {
			let slice = &self.cursor.bytes[self.cursor.index..=index_end];

			if slice == expect {
				self.cursor.advance(len);
				Some(slice)
			} else {
				None
			}
		} else {
			None
		}
	}

	pub const fn peek(&self) -> Option<u8> {
		self.cursor.first()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cursor<'a> {
	bytes: &'a [u8],
	index: usize,
}

impl<'a> Cursor<'a> {
	pub const fn new(bytes: &'a [u8]) -> Self {
		Self { bytes, index: 0 }
	}

	pub fn consume(&mut self) -> Option<u8> {
		let byte = self.first()?;
		self.advance(1);
		Some(byte)
	}

	pub fn consume_n(&mut self, n: usize) -> Option<&[u8]> {
		if self.in_bounds(self.index + n) {
			let bytes = &self.bytes[self.index..self.index + n];
			self.advance(n);
			Some(bytes)
		} else {
			None
		}
	}

	pub fn consume_while<P>(&mut self, predicate: P)
	where
		P: Fn(u8) -> bool,
	{
		while let Some(byte) = self.first() {
			if predicate(byte) {
				let _ = self.consume().expect("valid byte");
			} else {
				break;
			}
		}
	}

	pub fn consume_whitespaces(&mut self) {
		self.consume_while(|b| b.is_ascii_whitespace())
	}

	pub fn consume_non_whitespaces(&mut self) {
		self.consume_while(|b| !b.is_ascii_whitespace())
	}

	pub const fn first(&self) -> Option<u8> {
		self.nth(0)
	}

	pub const fn second(&self) -> Option<u8> {
		self.nth(1)
	}

	#[inline(always)]
	const fn nth(&self, n: usize) -> Option<u8> {
		let index = self.index + n;
		self.get(index)
	}

	#[inline(always)]
	const fn get(&self, index: usize) -> Option<u8> {
		if self.in_bounds(index) {
			Some(self.bytes[index])
		} else {
			None
		}
	}

	#[inline(always)]
	fn advance(&mut self, amount: usize) {
		self.advance_to(self.index + amount);
	}

	#[inline(always)]
	fn advance_to(&mut self, index: usize) {
		self.index = std::cmp::min(self.bytes.len(), index);
	}

	#[inline(always)]
	pub const fn index(&self) -> usize {
		self.index
	}

	#[inline(always)]
	pub const fn byte_pos(&self) -> BytePos {
		BytePos::from_usize(self.index)
	}

	#[inline(always)]
	fn index_mut(&mut self) -> &mut usize {
		&mut self.index
	}

	#[inline(always)]
	const fn in_bounds(&self, index: usize) -> bool {
		self.bytes.len() > index
	}

	#[inline(always)]
	pub const fn is_eof(&self) -> bool {
		self.index >= self.bytes.len()
	}
}
