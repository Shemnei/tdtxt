use std::fmt;

use chrono::prelude::*;

use crate::parse::{Parse, Parser};

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
				write!(f, "{} {}", completed.to_string(), created.to_string(),)
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
		Self::Completed { created: value.0.into(), completed: value.1.into() }
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
		let date1 = Date::parse_opt(parser).ok_or(DateCompoundParseError)?;
		let compound = match Date::parse_opt(parser) {
			Some(date2) => {
				Self::Completed { created: date2, completed: date1 }
			}
			None => Self::Created { created: date1 },
		};

		Ok(compound)
	}
}
