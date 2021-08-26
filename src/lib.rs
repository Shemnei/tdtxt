//! # Examples
//!
//! ```rust
//! use std::str::FromStr as _;
//!
//! use tdtxt::todo::{Todo, Date, State, Priority, DateCompound};
//!
//! let line = "x (A) 2016-05-20 2016-04-30 measure space for +chapelShelving @chapel due:2016-05-30";
//! let todo = Todo::from_str(line).unwrap();
//!
//! assert_eq!(todo.state(), Some(&State::Done));
//! assert_eq!(todo.priority(), Some(&Priority::A));
//! assert_eq!(todo.date_compound(), Some(&DateCompound::Completed { created: Date::ymd(2016, 4, 30), completed: Date::ymd(2016, 5, 20) }));
//! assert_eq!(todo.description().description(), "measure space for +chapelShelving @chapel due:2016-05-30");
//! assert_eq!(todo.description().projects(), vec!["chapelShelving"]);
//! assert_eq!(todo.description().contexts(), vec!["chapel"]);
//! assert_eq!(todo.description().custom(), vec![("due", "2016-05-30")]);
//! ```

#![allow(dead_code, rustdoc::private_intra_doc_links)]
#![deny(
    // Documentation
	// TODO: rustdoc::broken_intra_doc_links,
	// TODO: rustdoc::missing_crate_level_docs,
	// TODO: missing_docs,
	// TODO: clippy::missing_docs_in_private_items,

    // Other
	deprecated_in_future,
	exported_private_dependencies,
	future_incompatible,
	missing_copy_implementations,
	missing_debug_implementations,
	private_in_public,
	rust_2018_compatibility,
	rust_2018_idioms,
	trivial_casts,
	trivial_numeric_casts,
	unsafe_code,
	unstable_features,
	unused_import_braces,
	unused_qualifications,

	// clippy attributes
	clippy::missing_const_for_fn,
	clippy::redundant_pub_crate,
	clippy::use_self
)]
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_alias))]

pub(crate) mod parse {
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
			let index_end =
				self.cursor.index + if len > 0 { len - 1 } else { 0 };

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
}

pub mod todo {
	use std::cmp::Ordering;
	use std::convert::TryFrom;
	use std::fmt;
	use std::ops::{Deref, Range};

	use chrono::prelude::*;

	use crate::parse::{Cursor, Parse, Parser};

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
			let priority =
				parser.parse_alpha_upper().ok_or(PriorityParseError)?;
			let priority =
				Self::try_from(priority).map_err(|_| PriorityParseError)?;
			let _ = parser.expect_slice(b") ").ok_or(PriorityParseError)?;
			Ok(priority)
		}
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct Date {
		inner: chrono::Date<Local>,
	}

	impl Date {
		const DATE_FORMAT: &'static str = "%Y-%m-%d";

		pub fn ymd(year: i32, month: u32, day: u32) -> Self {
			Self { inner: Local.ymd(year, month, day) }
		}

		pub fn ymd_opt(year: i32, month: u32, day: u32) -> Option<Self> {
			let date = match Local.ymd_opt(year, month, day) {
				// TODO: additional "real" error
				chrono::LocalResult::None => return None,
				x => x.unwrap(),
			};

			Some(Self { inner: date })
		}

		pub fn today() -> Self {
			Self { inner: Local::today() }
		}
	}

	impl fmt::Display for Date {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.write_str(&self.inner.format(Self::DATE_FORMAT).to_string())
		}
	}

	impl From<(i32, u32, u32)> for Date {
		fn from(value: (i32, u32, u32)) -> Self {
			Self::ymd(value.0, value.1, value.2)
		}
	}

	// TODO: impl TryFrom

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct DateParseError;

	impl fmt::Display for DateParseError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.write_str("failed to parse date")
		}
	}

	impl std::error::Error for DateParseError {}

	impl Parse for Date {
		type Error = DateParseError;

		fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
			let y1 = parser.parse_digit().ok_or(DateParseError)?;
			let y2 = parser.parse_digit().ok_or(DateParseError)?;
			let y3 = parser.parse_digit().ok_or(DateParseError)?;
			let y4 = parser.parse_digit().ok_or(DateParseError)?;
			let _ = parser.expect_u8(b'-').ok_or(DateParseError)?;
			let m1 = parser.parse_digit().ok_or(DateParseError)?;
			let m2 = parser.parse_digit().ok_or(DateParseError)?;
			let _ = parser.expect_u8(b'-').ok_or(DateParseError)?;
			let d1 = parser.parse_digit().ok_or(DateParseError)?;
			let d2 = parser.parse_digit().ok_or(DateParseError)?;
			let _ = parser.expect_u8(b' ').ok_or(DateParseError)?;

			let year = (y1 as i32 * 1000)
				+ (y2 as i32 * 100)
				+ (y3 as i32 * 10)
				+ y4 as i32;
			let month = (m1 as u32 * 10) + m2 as u32;
			let day = (d1 as u32 * 10) + d2 as u32;

			let date = match Local.ymd_opt(year, month, day) {
				// TODO: additional "real" error
				chrono::LocalResult::None => return Err(DateParseError),
				x => x.unwrap(),
			};

			Ok(Self { inner: date })
		}
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub enum DateCompound {
		Created { created: Date },
		// TODO: assert created <= completed
		Completed { created: Date, completed: Date },
	}

	impl DateCompound {
		pub fn created<A>(created: A) -> Self
		where
			A: Into<Date>,
		{
			Self::Created { created: created.into() }
		}

		pub fn completed<A, B>(created: A, completed: B) -> Self
		where
			A: Into<Date>,
			B: Into<Date>,
		{
			Self::Completed {
				created: created.into(),
				completed: completed.into(),
			}
		}
	}

	impl fmt::Display for DateCompound {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match self {
				Self::Created { created } => f.write_str(&created.to_string()),
				Self::Completed { created, completed } => {
					write!(
						f,
						"{} {}",
						completed.to_string(),
						created.to_string(),
					)
				}
			}
		}
	}

	impl<A> From<A> for DateCompound
	where
		A: Into<Date>,
	{
		fn from(value: A) -> Self {
			Self::Created { created: value.into() }
		}
	}

	impl<A, B> From<(A, B)> for DateCompound
	where
		A: Into<Date>,
		B: Into<Date>,
	{
		fn from(value: (A, B)) -> Self {
			Self::Completed {
				created: value.0.into(),
				completed: value.1.into(),
			}
		}
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct DateCompoundParseError;

	impl fmt::Display for DateCompoundParseError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.write_str("failed to parse date compound")
		}
	}

	impl std::error::Error for DateCompoundParseError {}

	impl Parse for DateCompound {
		type Error = DateCompoundParseError;

		fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
			let date1 =
				Date::parse_opt(parser).ok_or(DateCompoundParseError)?;
			let compound = match Date::parse_opt(parser) {
				Some(date2) => {
					Self::Completed { created: date2, completed: date1 }
				}
				None => Self::Created { created: date1 },
			};

			Ok(compound)
		}
	}

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
				.map(|c| {
					(&self.raw[c.key.clone()], &self.raw[c.value.clone()])
				})
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
						let range =
							ProjectRange::new(word_start..cursor.index());
						projects.push(range);
					}

					// read context
					(Some(b'@'), Some(b)) if b != b' ' => {
						cursor.consume_while(|b| b != b' ');
						let range =
							ContextRange::new(word_start..cursor.index());
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

	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Todo {
		// TODO: remove pub
		pub(crate) state: Option<State>,
		pub(crate) priority: Option<Priority>,
		pub(crate) date_compound: Option<DateCompound>,
		pub(crate) description: Description,
	}

	impl Todo {
		pub fn build() -> TodoBuilder {
			Default::default()
		}

		pub const fn state(&self) -> Option<&State> {
			self.state.as_ref()
		}

		pub const fn priority(&self) -> Option<&Priority> {
			self.priority.as_ref()
		}

		pub const fn date_compound(&self) -> Option<&DateCompound> {
			self.date_compound.as_ref()
		}

		pub const fn description(&self) -> &Description {
			&self.description
		}
	}

	impl fmt::Display for Todo {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			let mut s: Vec<String> = Vec::with_capacity(4);

			if let Some(state) = self.state {
				s.push(state.to_string());
			}

			if let Some(priority) = self.priority {
				s.push(priority.to_string());
			}

			if let Some(date_compound) = self.date_compound {
				s.push(date_compound.to_string());
			}

			s.push(self.description.to_string());

			f.write_str(&s.join(" "))
		}
	}

	impl From<&str> for Description {
		fn from(value: &str) -> Self {
			Self::new(value)
		}
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct TodoParseError;

	impl fmt::Display for TodoParseError {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.write_str("failed to parse todo")
		}
	}

	impl std::error::Error for TodoParseError {}

	impl Parse for Todo {
		type Error = TodoParseError;

		fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
			let state = State::parse_opt(parser);
			let priority = Priority::parse_opt(parser);
			let date_compound = DateCompound::parse_opt(parser);
			let description =
				Description::parse(parser).map_err(|_| TodoParseError)?;

			let todo = Self { state, priority, date_compound, description };

			Ok(todo)
		}
	}

	impl std::str::FromStr for Todo {
		type Err = TodoParseError;

		fn from_str(s: &str) -> Result<Self, Self::Err> {
			let mut parser = Parser::new(s.as_bytes());
			Self::parse(&mut parser)
		}
	}

	#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
	pub struct TodoBuilder {
		state: Option<State>,
		priority: Option<Priority>,
		date_compound: Option<DateCompound>,
	}

	impl TodoBuilder {
		pub fn new() -> Self {
			let (state, priority, date_compound) = <_>::default();

			Self { state, priority, date_compound }
		}

		pub fn state(&mut self, state: State) -> &mut Self {
			self.state = Some(state);
			self
		}

		pub fn priority<P>(&mut self, priority: P) -> &mut Self
		where
			P: Into<Priority>,
		{
			self.priority = Some(priority.into());
			self
		}

		pub fn date_compound<D>(&mut self, date_compound: D) -> &mut Self
		where
			D: Into<DateCompound>,
		{
			self.date_compound = Some(date_compound.into());
			self
		}

		pub fn build<D>(&mut self, description: D) -> Todo
		where
			D: Into<Description>,
		{
			Todo {
				state: self.state,
				priority: self.priority,
				date_compound: self.date_compound,
				description: description.into(),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::parse::*;
	use crate::todo::{Date, *};

	#[test]
	fn todo_display() {
		let todo = Todo {
			state: Some(State::Done),
			priority: Some(Priority::H),
			date_compound: None,
			description: Description::new("Hello World"),
		};

		assert_eq!(todo.to_string(), "x (H) Hello World");
	}

	#[test]
	fn todo_parse() {
		let input = b"x ";
		let mut parser = Parser::new(input);

		assert_eq!(State::parse(&mut parser), Ok(State::Done));

		let input = b"(H) ";
		let mut parser = Parser::new(input);

		assert_eq!(Priority::parse(&mut parser), Ok(Priority::H));

		let input = b"2020-01-01 ";
		let mut parser = Parser::new(input);

		assert_eq!(Date::parse(&mut parser), Ok(Date::ymd(2020, 01, 01)));

		let input = b"1234-07-16 ";
		let mut parser = Parser::new(input);

		let d = DateCompound::Created { created: Date::ymd(1234, 07, 16) };
		assert_eq!(DateCompound::parse(&mut parser), Ok(d));

		let input = b"2000-01-01 1970-01-01 ";
		let mut parser = Parser::new(input);

		let d = DateCompound::Completed {
			created: Date::ymd(1970, 01, 01),
			completed: Date::ymd(2000, 01, 01),
		};
		assert_eq!(DateCompound::parse(&mut parser), Ok(d));

		let input = b"Hello World";
		let mut parser = Parser::new(input);

		assert_eq!(
			Description::parse(&mut parser),
			Ok(Description::new("Hello World"))
		);

		let input = b"Hello World\nfoo bar";
		let mut parser = Parser::new(input);

		assert_eq!(
			Description::parse(&mut parser),
			Ok(Description::new("Hello World"))
		);
		assert_eq!(
			Description::parse(&mut parser),
			Ok(Description::new("foo bar"))
		);

		let input = b"x (Z) 2020-01-01 Hello World";
		let mut parser = Parser::new(input);

		let todo = Todo {
			state: Some(State::Done),
			priority: Some(Priority::Z),
			date_compound: Some(DateCompound::Created {
				created: Date::ymd(2020, 01, 01),
			}),
			description: Description::new("Hello World"),
		};

		assert_eq!(Todo::parse(&mut parser), Ok(todo));
	}

	#[test]
	fn todo_example() {
		let input = b"(A) Thank Mom for the meatballs @phone
(B) Schedule Goodwill pickup +GarageSale @phone
Post signs around the neighborhood +GarageSale
@GroceryStore Eskimo pies";
		let mut parser = Parser::new(input);

		let todo = Todo::build()
			.priority(Priority::A)
			.build("Thank Mom for the meatballs @phone");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		let todo = Todo::build()
			.priority(Priority::B)
			.build("Schedule Goodwill pickup +GarageSale @phone");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		let todo = Todo::build()
			.build("Post signs around the neighborhood +GarageSale");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		let todo = Todo::build().build("@GroceryStore Eskimo pies");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	#[test]
	fn todo_rule1() {
		let input = b"(A) Call Mom";
		let mut parser = Parser::new(input);

		let todo = Todo::build().priority(Priority::A).build("Call Mom");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));

		let input = b"Really gotta call Mom (A) @phone @someday
(b) Get back to the boss
(B)->Submit TPS report";
		let mut parser = Parser::new(input);

		let todo =
			Todo::build().build("Really gotta call Mom (A) @phone @someday");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		let todo = Todo::build().build("(b) Get back to the boss");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		let todo = Todo::build().build("(B)->Submit TPS report");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	#[test]
	fn todo_rule2() {
		let input = b"2011-03-02 Document +TodoTxt task format
(A) 2011-03-02 Call Mom";
		let mut parser = Parser::new(input);

		let todo = Todo::build()
			.date_compound(DateCompound::Created {
				created: Date::ymd(2011, 03, 02),
			})
			.build("Document +TodoTxt task format");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		let todo = Todo::build()
			.priority(Priority::A)
			.date_compound(DateCompound::Created {
				created: Date::ymd(2011, 03, 02),
			})
			.build("Call Mom");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));

		let input = b"(A) Call Mom 2011-03-02";
		let mut parser = Parser::new(input);

		let todo =
			Todo::build().priority(Priority::A).build("Call Mom 2011-03-02");
		assert_eq!(Todo::parse(&mut parser), Ok(todo));

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	#[test]
	fn todo_rule3() {
		let input =
			b"(A) Call Mom +Family +PeaceLoveAndHappiness @iphone @phone";
		let mut parser = Parser::new(input);

		let todo_should = Todo::build()
			.priority(Priority::A)
			.build("Call Mom +Family +PeaceLoveAndHappiness @iphone @phone");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		let projects_should = vec!["Family", "PeaceLoveAndHappiness"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should = vec!["iphone", "phone"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = Vec::new();
		assert_eq!(todo_is.description.custom(), custom_should);

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));

		let input = b"Email SoAndSo at soandso@example.com";
		let mut parser = Parser::new(input);

		let todo_should =
			Todo::build().build("Email SoAndSo at soandso@example.com");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		let projects_should: Vec<&str> = Vec::new();
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = Vec::new();
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = Vec::new();
		assert_eq!(todo_is.description.custom(), custom_should);

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));

		let input = b"Learn how to add 2+2";
		let mut parser = Parser::new(input);

		let todo_should = Todo::build().build("Learn how to add 2+2");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		let projects_should: Vec<&str> = Vec::new();
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = Vec::new();
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = Vec::new();
		assert_eq!(todo_is.description.custom(), custom_should);

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	#[test]
	fn todo_parse_full() {
		let input =
            b"x (J) 1990-01-01 1980-01-01 Wait ten year @home for +century_waiting author:me";
		let mut parser = Parser::new(input);

		let todo_should = Todo::build()
			.state(State::Done)
			.priority(Priority::J)
			.date_compound(DateCompound::Completed {
				created: Date::ymd(1980, 01, 01),
				completed: Date::ymd(1990, 01, 01),
			})
			.build("Wait ten year @home for +century_waiting author:me");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(todo_is.to_string().as_bytes(), input);
		let projects_should: Vec<&str> = vec!["century_waiting"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["home"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![("author", "me")];
		assert_eq!(todo_is.description.custom(), custom_should);

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	#[test]
	fn todo_parse_edge() {
		let input = b"add + some more not::valid @home @work";
		let mut parser = Parser::new(input);

		let todo_should =
			Todo::build().build("add + some more not::valid @home @work");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		let projects_should: Vec<&str> = Vec::new();
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["home", "work"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = Vec::new();
		assert_eq!(todo_is.description.custom(), custom_should);

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	// http://todotxt.org/todo.txt
	#[test]
	fn todo_parse_example() {
		let input = b"(A) Call Mom @Phone +Family
(A) Schedule annual checkup +Health
(B) Outline chapter 5 +Novel @Computer
(C) Add cover sheets @Office +TPSReports
Plan backyard herb garden @Home
Pick up milk @GroceryStore
Research self-publishing services +Novel @Computer
x Download Todo.txt mobile app @Phone";
		let mut parser = Parser::new(input);

		let todo_should = Todo::build()
			.priority(Priority::A)
			.build("Call Mom @Phone +Family");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"(A) Call Mom @Phone +Family"
		);
		let projects_should: Vec<&str> = vec!["Family"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["Phone"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should = Todo::build()
			.priority(Priority::A)
			.build("Schedule annual checkup +Health");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"(A) Schedule annual checkup +Health"
		);
		let projects_should: Vec<&str> = vec!["Health"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec![];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should = Todo::build()
			.priority(Priority::B)
			.build("Outline chapter 5 +Novel @Computer");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"(B) Outline chapter 5 +Novel @Computer"
		);
		let projects_should: Vec<&str> = vec!["Novel"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["Computer"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should = Todo::build()
			.priority(Priority::C)
			.build("Add cover sheets @Office +TPSReports");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"(C) Add cover sheets @Office +TPSReports"
		);
		let projects_should: Vec<&str> = vec!["TPSReports"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["Office"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should =
			Todo::build().build("Plan backyard herb garden @Home");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"Plan backyard herb garden @Home"
		);
		let projects_should: Vec<&str> = vec![];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["Home"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should = Todo::build().build("Pick up milk @GroceryStore");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"Pick up milk @GroceryStore"
		);
		let projects_should: Vec<&str> = vec![];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["GroceryStore"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should = Todo::build()
			.build("Research self-publishing services +Novel @Computer");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"Research self-publishing services +Novel @Computer"
		);
		let projects_should: Vec<&str> = vec!["Novel"];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["Computer"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		let todo_should = Todo::build()
			.state(State::Done)
			.build("Download Todo.txt mobile app @Phone");
		let todo_is = Todo::parse(&mut parser);
		assert_eq!(todo_is, Ok(todo_should));
		let todo_is = todo_is.unwrap();
		assert_eq!(
			todo_is.to_string().as_bytes(),
			b"x Download Todo.txt mobile app @Phone"
		);
		let projects_should: Vec<&str> = vec![];
		assert_eq!(todo_is.description.projects(), projects_should);
		let contexts_should: Vec<&str> = vec!["Phone"];
		assert_eq!(todo_is.description.contexts(), contexts_should);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(todo_is.description.custom(), custom_should);

		assert_eq!(Todo::parse(&mut parser), Err(TodoParseError));
	}

	#[test]
	fn priority_ord() {
		assert!(Priority::A > Priority::B);
		assert!(Priority::A == Priority::A);
		assert!(Priority::Z < Priority::A);
	}
}
