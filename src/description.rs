use std::fmt;
use std::ops::{Deref, Index};

use crate::parse::{Cursor, Parse, Parser};
use crate::span::ByteSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectRange {
	full: ByteSpan,
	name: ByteSpan,
}

impl ProjectRange {
	pub const fn new(full: ByteSpan) -> Self {
		let name = full.offset_low(1);

		Self { full, name }
	}

	pub const fn full(&self) -> &ByteSpan {
		&self.full
	}

	pub const fn project(&self) -> &ByteSpan {
		&self.name
	}

	pub fn index<'a, 'b>(&'a self, s: &'b str) -> &'b str {
		Index::index(s, self.name)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextRange {
	full: ByteSpan,
	name: ByteSpan,
}

impl ContextRange {
	pub const fn new(full: ByteSpan) -> Self {
		let name = full.offset_low(1);

		Self { full, name }
	}

	pub const fn full(&self) -> &ByteSpan {
		&self.full
	}

	pub const fn context(&self) -> &ByteSpan {
		&self.name
	}

	pub fn index<'a, 'b>(&'a self, s: &'b str) -> &'b str {
		Index::index(s, self.name)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomRange {
	full: ByteSpan,
	key: ByteSpan,
	value: ByteSpan,
}

impl CustomRange {
	pub fn new(key: ByteSpan, value: ByteSpan) -> Self {
		let full = key.union(&value);

		Self { full, key, value }
	}

	pub const fn full(&self) -> &ByteSpan {
		&self.full
	}

	pub const fn key(&self) -> &ByteSpan {
		&self.key
	}

	pub const fn value(&self) -> &ByteSpan {
		&self.value
	}

	pub fn index<'a, 'b>(&'a self, s: &'b str) -> (&'b str, &'b str) {
		(Index::index(s, self.key), Index::index(s, self.value))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description {
	raw: String,
	projects: Vec<ProjectRange>,
	contexts: Vec<ContextRange>,
	custom: Vec<CustomRange>,
}

impl Description {
	pub fn new<S>(s: S) -> Self
	where
		S: ToString,
	{
		let raw: String = s.to_string();
		let (projects, contexts, custom) = Self::index(&raw);

		Self { raw, projects, contexts, custom }
	}

	pub fn description(&self) -> &str {
		&self.raw
	}

	pub fn projects(&self) -> ProjectIter<'_> {
		ProjectIter::new(self)
	}

	pub fn contexts(&self) -> ContextIter<'_> {
		ContextIter::new(self)
	}

	pub fn custom(&self) -> CustomIter<'_> {
		CustomIter::new(self)
	}

	// project: \+[^ ]+
	// context: \@[^ ]+
	// custom : [^ ]+\:[^ ]+
	//
	fn index(
		s: &str,
	) -> (Vec<ProjectRange>, Vec<ContextRange>, Vec<CustomRange>) {
		const fn is_key_value_byte(byte: u8) -> bool {
			!byte.is_ascii_whitespace() && byte != b':'
		}

		let mut projects = Vec::new();
		let mut contexts = Vec::new();
		let mut custom = Vec::new();

		let mut cursor = Cursor::new(s.as_bytes());

		while !cursor.is_eof() {
			// reset word boundry (todo: handle unicode whitespace)
			cursor.consume_whitespaces();
			let word_start = cursor.byte_pos();

			match (cursor.first(), cursor.second()) {
				// read project
				(Some(b'+'), Some(b)) if !b.is_ascii_whitespace() => {
					cursor.consume_non_whitespaces();
					let span = ByteSpan::new(word_start, cursor.byte_pos());
					let range = ProjectRange::new(span);
					projects.push(range);
				}

				// read context
				(Some(b'@'), Some(b)) if !b.is_ascii_whitespace() => {
					cursor.consume_non_whitespaces();
					let span = ByteSpan::new(word_start, cursor.byte_pos());
					let range = ContextRange::new(span);
					contexts.push(range);
				}

				// try read custom tag
				(Some(_), Some(b)) if !b.is_ascii_whitespace() => {
					cursor.consume_while(is_key_value_byte);

					if let Some(b':') = cursor.first() {
						let key_span =
							ByteSpan::new(word_start, cursor.byte_pos());

						assert_eq!(cursor.consume(), Some(b':'));

						if key_span.len() > 0
							&& matches!(cursor.first(), Some(b) if is_key_value_byte(b))
						{
							let value_start = cursor.byte_pos();
							cursor.consume_while(is_key_value_byte);

							let value_span =
								ByteSpan::new(value_start, cursor.byte_pos());

							if value_span.len() == 0
								|| matches!(cursor.first(), Some(b) if !b.is_ascii_whitespace())
							{
								// Tag value does not end in eof or white space; invalid
								// Keep word boundaries intact
								cursor.consume_non_whitespaces();
							} else {
								let range =
									CustomRange::new(key_span, value_span);
								custom.push(range);
							}
						} else {
							// Keep word boundaries intact
							cursor.consume_non_whitespaces();
						}
					} else {
						// Keep word boundaries intact
						cursor.consume_non_whitespaces();
					}
				}

				// skip word
				(Some(_), _) => {
					cursor.consume_non_whitespaces();
				}

				// exit on eof
				(None, _) => break,
			}
		}

		(projects, contexts, custom)
	}
}

impl fmt::Display for Description {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.raw)
	}
}

impl Deref for Description {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.raw
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseDescriptionError;

impl fmt::Display for ParseDescriptionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("failed to parse description")
	}
}

impl std::error::Error for ParseDescriptionError {}

impl Parse for Description {
	type Error = ParseDescriptionError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let description =
			parser.parse_until(b'\n').ok_or(ParseDescriptionError)?;
		let description = std::str::from_utf8(description)
			.map_err(|_| ParseDescriptionError)?;
		let description = Self::new(description);

		// consume possible new line
		let _ = parser.parse_u8();

		Ok(description)
	}
}

macro_rules! simple_iter {
	( $name:ident => $range:ty, $rangevar:ident, $item:ty) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		pub struct $name<'a> {
			description: &'a str,
			ranges: &'a [$range],
			ranges_idx: usize,
		}

		impl<'a> $name<'a> {
			fn new(description: &'a Description) -> Self {
				Self {
					description: &description.raw,
					ranges: &description.$rangevar,
					ranges_idx: 0,
				}
			}
		}

		impl<'a> Iterator for $name<'a> {
			type Item = $item;

			fn next(&mut self) -> Option<Self::Item> {
				let range = self.ranges.get(self.ranges_idx)?;
				self.ranges_idx += 1;

				Some(range.index(&self.description))
			}
		}
	};
}

simple_iter!(ProjectIter => ProjectRange, projects, &'a str);
simple_iter!(ContextIter => ContextRange, contexts, &'a str);
simple_iter!(CustomIter => CustomRange, custom, (&'a str, &'a str));
