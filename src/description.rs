use std::fmt;
use std::ops::{Deref, Range};

use crate::parse::{Cursor, Parse, Parser};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectRange {
	full: Range<usize>,
	name: Range<usize>,
}

impl ProjectRange {
	pub const fn new(full: Range<usize>) -> Self {
		let name = full.start + 1..full.end;

		Self { full, name }
	}

	pub const fn full(&self) -> &Range<usize> {
		&self.full
	}

	pub const fn project(&self) -> &Range<usize> {
		&self.name
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextRange {
	full: Range<usize>,
	name: Range<usize>,
}

impl ContextRange {
	pub const fn new(full: Range<usize>) -> Self {
		let name = full.start + 1..full.end;

		Self { full, name }
	}

	pub const fn full(&self) -> &Range<usize> {
		&self.full
	}

	pub const fn context(&self) -> &Range<usize> {
		&self.name
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomRange {
	full: Range<usize>,
	key: Range<usize>,
	value: Range<usize>,
}

impl CustomRange {
	pub const fn new(key: Range<usize>, value: Range<usize>) -> Self {
		let full = key.start..value.end;

		Self { full, key, value }
	}

	pub const fn full(&self) -> &Range<usize> {
		&self.full
	}

	pub const fn key(&self) -> &Range<usize> {
		&self.key
	}

	pub const fn value(&self) -> &Range<usize> {
		&self.value
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

	pub fn projects(&self) -> Vec<&str> {
		self.projects.iter().map(|p| &self.raw[p.name.clone()]).collect()
	}

	pub fn contexts(&self) -> Vec<&str> {
		self.contexts.iter().map(|c| &self.raw[c.name.clone()]).collect()
	}

	pub fn custom(&self) -> Vec<(&str, &str)> {
		self.custom
			.iter()
			.map(|c| (&self.raw[c.key.clone()], &self.raw[c.value.clone()]))
			.collect()
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

		let mut word_start = 0;
		while !cursor.is_eof() {
			match (cursor.first(), cursor.second()) {
				// reset word boundry
				(Some(b' '), _) => {
					assert_eq!(cursor.consume(), Some(b' '));
					word_start = cursor.index();
				}

				// read project
				(Some(b'+'), Some(b)) if b != b' ' => {
					cursor.consume_while(|b| b != b' ');
					let range = ProjectRange::new(word_start..cursor.index());
					projects.push(range);
				}

				// read context
				(Some(b'@'), Some(b)) if b != b' ' => {
					cursor.consume_while(|b| b != b' ');
					let range = ContextRange::new(word_start..cursor.index());
					contexts.push(range);
				}

				// try read custom tag
				(Some(a), Some(b)) if a != b' ' && b != b' ' => {
					cursor.consume_while(|b| b != b' ' && b != b':');
					if let Some(b':') = cursor.first() {
						let key_range = word_start..cursor.index();
						assert_eq!(cursor.consume(), Some(b':'));
						match cursor.first() {
							Some(b':') | Some(b' ') => {
								cursor.consume_while(|b| b != b' ');
							}
							_ => {
								let value_start = cursor.index();
								cursor.consume_while(|b| b != b' ');
								let range = CustomRange::new(
									key_range,
									value_start..cursor.index(),
								);
								custom.push(range);
							}
						}
					}
				}

				// skip normal word
				(Some(_), _) => {
					cursor.consume_while(|b| b != b' ');
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
pub struct DescriptionParseError;

impl fmt::Display for DescriptionParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("failed to parse description")
	}
}

impl std::error::Error for DescriptionParseError {}

impl Parse for Description {
	type Error = DescriptionParseError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let description =
			parser.parse_until(b'\n').ok_or(DescriptionParseError)?;
		let description = std::str::from_utf8(description)
			.map_err(|_| DescriptionParseError)?;
		let description = Self::new(description);

		// consume possible new line
		let _ = parser.parse_u8();

		Ok(description)
	}
}
