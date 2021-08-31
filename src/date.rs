use std::fmt;

#[cfg(feature = "chrono")]
use chrono::prelude::*;

use crate::parse::{Parse, Parser};

/// A very basic date type used when feature `chrono` is not active.
#[cfg(not(feature = "chrono"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleDate {
	/// Year of the date.
	year: i16,
	/// One-indexed month of the date.
	month: u8,
	/// One-indexed day of the date.
	day: u8,
}

#[cfg(not(feature = "chrono"))]
impl SimpleDate {
	/// Creates a new date.
	///
	/// # Panics
	///
	/// Can panic if the month/day are not within the limits.
	///
	/// - Month: 1-12
	/// - Day:   1-31
	pub fn ymd(year: i16, month: u8, day: u8) -> Self {
		assert!((1..=12).contains(&month), "month must be between 1-12");
		assert!((1..=31).contains(&day), "day must be between 1-31");

		Self { year, month, day }
	}

	/// Creates a new date. Will return `None` if the date creation would have
	/// failed.
	///
	/// For more information about what could fail see: [`Self::ymd`].
	pub fn ymd_opt(year: i16, month: u8, day: u8) -> Option<Self> {
		if (1..=12).contains(&month) && (1..=31).contains(&day) {
			Some(Self { year, month, day })
		} else {
			None
		}
	}
}

#[cfg(not(feature = "chrono"))]
impl fmt::Display for SimpleDate {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:04}-{:02}-{:02}", &self.year, &self.month, &self.day)
	}
}

/// A simple date structure, which represents the date in the format
/// `yyyy-mm-dd`.
///
/// # Notes
///
/// The inner/backing type is depended on the feature `chrono`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Date {
	/// Inner backing type.
	#[cfg(feature = "chrono")]
	inner: chrono::Date<Local>,

	/// Inner backing type.
	#[cfg(not(feature = "chrono"))]
	inner: SimpleDate,
}

impl Date {
	/// The format for printing the date when feature `chrono` is active.
	#[cfg(feature = "chrono")]
	const DATE_FORMAT: &'static str = "%Y-%m-%d";

	/// Creates a new date.
	///
	/// # Panics
	///
	/// Can panic if the date is invalid.
	#[cfg_attr(
		feature = "chrono",
		doc = "For more information see the relevant backing \
		       implementation:[`chrono::Local::ymd`]"
	)]
	pub fn ymd(year: i16, month: u8, day: u8) -> Self {
		#[cfg(feature = "chrono")]
		{
			Self { inner: Local.ymd(year as i32, month as u32, day as u32) }
		}

		#[cfg(not(feature = "chrono"))]
		{
			Self { inner: SimpleDate::ymd(year, month, day) }
		}
	}

	/// Creates a new date. Returns `None` when the date creation would have
	/// failed.
	#[cfg_attr(
		feature = "chrono",
		doc = "\n# Notes\nFor more information see the relevant backing \
		       implementation:[`chrono::Local::ymd_opt`]
	"
	)]
	pub fn ymd_opt(year: i16, month: u8, day: u8) -> Option<Self> {
		#[cfg(feature = "chrono")]
		{
			let date = Local
				.ymd_opt(year as i32, month as u32, day as u32)
				.earliest()?;

			Some(Self { inner: date })
		}

		#[cfg(not(feature = "chrono"))]
		{
			Some(Self { inner: SimpleDate::ymd_opt(year, month, day)? })
		}
	}

	/// Returns a `Date` which corresponds to the current date.
	#[cfg(feature = "chrono")]
	pub fn today() -> Self {
		Self { inner: Local::today() }
	}
}

impl fmt::Display for Date {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		#[cfg(feature = "chrono")]
		{
			f.write_str(&self.inner.format(Self::DATE_FORMAT).to_string())
		}

		#[cfg(not(feature = "chrono"))]
		{
			fmt::Display::fmt(&self.inner, f)
		}
	}
}

crate::parse_error!(ParseDateError: "date");

impl Parse for Date {
	type Error = ParseDateError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let y1 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let y2 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let y3 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let y4 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let _ = parser.expect_u8(b'-').ok_or_else(ParseDateError::default)?;
		let m1 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let m2 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let _ = parser.expect_u8(b'-').ok_or_else(ParseDateError::default)?;
		let d1 = parser.parse_digit().ok_or_else(ParseDateError::default)?;
		let d2 = parser.parse_digit().ok_or_else(ParseDateError::default)?;

		let year = (y1 as i16 * 1000)
			+ (y2 as i16 * 100)
			+ (y3 as i16 * 10)
			+ y4 as i16;
		let month = (m1 * 10) + m2;
		let day = (d1 * 10) + d2;

		Self::ymd_opt(year, month, day).ok_or_else(ParseDateError::default)
	}
}

crate::impl_fromstr!(Date);

#[cfg(feature = "serde")]
impl serde::Serialize for Date {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

#[cfg(feature = "serde")]
struct DateVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for DateVisitor {
	type Value = Date;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str("a date with the format of 'yyyy-mm-dd'")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		std::str::FromStr::from_str(v).map_err(serde::de::Error::custom)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for Date {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_str(DateVisitor)
	}
}

/// Represents the attached dates a [`Task`](`crate::Task`) can have.
///
/// The dates must be given in the format `yyyy-mm-dd`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DateCompound {
	/// A single date on which the task was created.
	Created {
		/// Creation date.
		created: Date,
	},
	/// Two dates, a completion date and a creation date.
	Completed {
		/// Creation date.
		created: Date,
		/// Completion date.
		completed: Date,
	},
}

impl DateCompound {
	/// Creates a new date compound for a single created date
	/// ([`DateCompound::Created`]).
	pub fn created<A>(created: A) -> Self
	where
		A: Into<Date>,
	{
		Self::Created { created: created.into() }
	}

	/// Creates a new date compound for a created and completion date
	/// ([`DateCompound::Completed`]).
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

crate::parse_error!(ParseDateCompoundError: "date compound");

impl Parse for DateCompound {
	type Error = ParseDateCompoundError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		let date1 = Date::parse_opt(parser)
			.ok_or_else(ParseDateCompoundError::default)?;

		let mut p_copy = *parser;

		if p_copy.expect_whitespace().is_some() {
			if let Some(date2) = Date::parse_opt(&mut p_copy) {
				// Check if eof or white space; if not it is a single date
				if p_copy
					.peek()
					.map(|c| c.is_ascii_whitespace())
					.unwrap_or(true)
				{
					*parser = p_copy;

					return Ok(Self::Completed {
						created: date2,
						completed: date1,
					});
				}
			}
		}

		Ok(Self::Created { created: date1 })
	}
}

crate::impl_fromstr!(DateCompound);
