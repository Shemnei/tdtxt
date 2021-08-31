use std::fmt;
use std::ops::{Deref, Index};

use crate::parse::{Cursor, Parse, Parser};
use crate::span::{BytePos, ByteSpan};

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

/// Represents the description part of a [`Task`](`crate::Task`).
///
/// Components like projects, contexts and custom tags are all implemented as
/// byte range indices into the raw description text. This is done to avoid
/// unnecessary allocations which in turn reduces the memory footprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description {
	/// The whole text of the description.
	raw: String,
	/// Byte indices into [`Self::raw`] representing projects (e.g. `+project`);
	projects: Vec<ProjectRange>,
	/// Byte indices into [`Self::raw`] representing contexts (e.g. `@context`);
	contexts: Vec<ContextRange>,
	/// Byte indices into [`Self::raw`] representing custom tags (e.g.
	/// `key:value`);
	custom: Vec<CustomRange>,
}

impl Description {
	/// Creates a new description from `s`.
	///
	/// During this process all projects, contexts and custom tags will be
	/// located.
	pub fn new<S>(s: S) -> Self
	where
		S: ToString,
	{
		let raw: String = s.to_string();
		let (projects, contexts, custom) = Self::index(&raw);

		Self { raw, projects, contexts, custom }
	}

	/// Returns the text of the whole description.
	pub fn description(&self) -> &str {
		&self.raw
	}

	/// Returns an iterator of all projects found within the description.
	pub fn projects(&self) -> ProjectIter<'_> {
		ProjectIter::new(self)
	}

	/// Returns an iterator of all contexts found within the description.
	pub fn contexts(&self) -> ContextIter<'_> {
		ContextIter::new(self)
	}

	/// Returns an iterator of all custom tags found within the description.
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
					projects.push(Self::read_project(&mut cursor, word_start));
				}

				// read context
				(Some(b'@'), Some(b)) if !b.is_ascii_whitespace() => {
					contexts.push(Self::read_context(&mut cursor, word_start));
				}

				// try read custom tag
				(Some(_), Some(b)) if !b.is_ascii_whitespace() => {
					if let Some(range) =
						Self::read_custom(&mut cursor, word_start)
					{
						custom.push(range);
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

			// TODO: check and warn if not at word boundry
			debug_assert!(cursor
				.first()
				.map(|b| b.is_ascii_whitespace())
				.unwrap_or(true));
		}

		(projects, contexts, custom)
	}

	fn read_project(
		cursor: &mut Cursor<'_>,
		word_start: BytePos,
	) -> ProjectRange {
		cursor.consume_non_whitespaces();
		let span = ByteSpan::new(word_start, cursor.byte_pos());
		ProjectRange::new(span)
	}

	fn read_context(
		cursor: &mut Cursor<'_>,
		word_start: BytePos,
	) -> ContextRange {
		cursor.consume_non_whitespaces();
		let span = ByteSpan::new(word_start, cursor.byte_pos());
		ContextRange::new(span)
	}

	fn read_custom(
		cursor: &mut Cursor<'_>,
		word_start: BytePos,
	) -> Option<CustomRange> {
		const fn is_key_value_byte(byte: u8) -> bool {
			!byte.is_ascii_whitespace() && byte != b':'
		}

		cursor.consume_while(is_key_value_byte);

		if let Some(b':') = cursor.first() {
			let key_span = ByteSpan::new(word_start, cursor.byte_pos());

			debug_assert_eq!(cursor.consume(), Some(b':'));

			if key_span.len() > 0
				&& matches!(cursor.first(), Some(b) if is_key_value_byte(b))
			{
				let value_start = cursor.byte_pos();
				cursor.consume_while(is_key_value_byte);

				let value_span = ByteSpan::new(value_start, cursor.byte_pos());

				if !(value_span.len() == 0
					|| matches!(cursor.first(), Some(b) if !b.is_ascii_whitespace()))
				{
					return Some(CustomRange::new(key_span, value_span));
				}
			}
		}

		None
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

crate::parse_error!(ParseDescriptionError: "description");

impl Parse for Description {
	type Error = ParseDescriptionError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let description = parser
			.parse_until(b'\n')
			.ok_or_else(ParseDescriptionError::default)?;
		let description = std::str::from_utf8(description)
			.map_err(|_| ParseDescriptionError::default())?;
		let description = Self::new(description);

		// consume possible new line
		let _ = parser.parse_u8();

		Ok(description)
	}
}

crate::impl_fromstr!(Description);

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

#[cfg(feature = "serde")]
impl serde::Serialize for Description {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.raw)
	}
}

#[cfg(feature = "serde")]
struct DescriptionVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for DescriptionVisitor {
	type Value = Description;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str("a description")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		std::str::FromStr::from_str(v).map_err(serde::de::Error::custom)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for Description {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_str(DescriptionVisitor)
	}
}
