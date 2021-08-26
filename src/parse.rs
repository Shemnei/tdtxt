pub trait Parse: Sized {
	type Error: std::error::Error;

	// TODO: some indication weather we reached EOF.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parser<'a> {
	cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
	pub const fn new(bytes: &'a [u8]) -> Self {
		Self { cursor: Cursor::new(bytes) }
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

	pub fn expect_slice<S>(&mut self, expect: S) -> Option<&[u8]>
	where
		S: AsRef<[u8]>,
	{
		let expect = expect.as_ref();
		let len = expect.len();
		let index_end = self.cursor.index + if len > 0 { len - 1 } else { 0 };

		if self.cursor.in_bounds(index_end) {
			let slice = &self.cursor.bytes[self.cursor.index..=index_end];
			let diff = slice.iter().zip(expect).find(|(a, b)| a != b);

			if diff.is_none() {
				self.cursor.advance(len);
				Some(slice)
			} else {
				None
			}
		} else {
			None
		}
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