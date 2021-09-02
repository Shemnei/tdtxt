use std::fmt;

use crate::date::DateCompound;
use crate::description::Description;
use crate::parse::{Parse, Parser};
use crate::priority::Priority;
use crate::state::State;

/// Represents the whole task.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Task {
	/// Optional state of the task.
	pub state: State,

	/// Optional priority of the task.
	#[cfg_attr(
		feature = "serde",
		serde(skip_serializing_if = "Option::is_none", default)
	)]
	pub priority: Option<Priority>,

	/// Optional associated special dates for the task.
	#[cfg_attr(
		feature = "serde",
		serde(flatten, skip_serializing_if = "Option::is_none", default)
	)]
	pub date_compound: Option<DateCompound>,

	/// Description of the task.
	pub description: Description,
}

impl Task {
	/// Creates a new builder for a task.
	pub fn build() -> TaskBuilder {
		Default::default()
	}

	/// Returns the state of the task.
	pub const fn state(&self) -> &State {
		&self.state
	}

	/// Returns the priority of the task.
	pub const fn priority(&self) -> Option<&Priority> {
		self.priority.as_ref()
	}

	/// Returns the date compound of the task.
	pub const fn date_compound(&self) -> Option<&DateCompound> {
		self.date_compound.as_ref()
	}

	/// Returns the description of the task.
	pub const fn description(&self) -> &Description {
		&self.description
	}
}

impl fmt::Display for Task {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s: Vec<String> = Vec::with_capacity(4);

		if self.state != State::Open {
			s.push(self.state.to_string());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseTaskError;

impl fmt::Display for ParseTaskError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("failed to parse task")
	}
}

impl std::error::Error for ParseTaskError {}

impl Parse for Task {
	type Error = ParseTaskError;

	fn parse(parser: &mut Parser<'_>) -> Result<Self, Self::Error> {
		macro_rules! try_parse {
			( $parser:ident : $ty:ty ) => {{
				let mut p_copy = *parser;

				if let Some(ty) = <$ty>::parse_opt(&mut p_copy) {
					if p_copy.is_eof() || p_copy.expect_whitespace().is_some()
					{
						*parser = p_copy;
						Some(ty)
					} else {
						None
					}
				} else {
					None
				}
			}};
		}

		let state = try_parse!(parser: State).unwrap_or_default();
		let priority = try_parse!(parser: Priority);
		let date_compound = try_parse!(parser: DateCompound);

		let description =
			Description::parse(parser).map_err(|_| ParseTaskError)?;

		let task = Self { state, priority, date_compound, description };

		Ok(task)
	}
}

impl std::str::FromStr for Task {
	type Err = ParseTaskError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut parser = Parser::new(s.as_bytes());
		Self::parse(&mut parser)
	}
}

/// A builder for a task.
///
/// All components implement `Copy`, meaning the builder can be used to build
/// multiple tasks without being consumed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskBuilder {
	state: Option<State>,
	priority: Option<Priority>,
	date_compound: Option<DateCompound>,
}

impl TaskBuilder {
	/// Creates a new instance of the builder.
	pub fn new() -> Self {
		let (state, priority, date_compound) = <_>::default();

		Self { state, priority, date_compound }
	}

	/// Sets the state for the task.
	pub fn state(&mut self, state: State) -> &mut Self {
		self.state = Some(state);
		self
	}

	/// Sets the priority for the task.
	pub fn priority<P>(&mut self, priority: P) -> &mut Self
	where
		P: Into<Priority>,
	{
		self.priority = Some(priority.into());
		self
	}

	/// Sets the date compound for the task.
	pub fn date_compound<D>(&mut self, date_compound: D) -> &mut Self
	where
		D: Into<DateCompound>,
	{
		self.date_compound = Some(date_compound.into());
		self
	}

	/// Creates a task from the builder.
	///
	/// # Notes
	///
	/// If no priority was set it will use the default implementation for it.
	pub fn build<D>(&mut self, description: D) -> Task
	where
		D: Into<Description>,
	{
		Task {
			state: self.state.unwrap_or_default(),
			priority: self.priority,
			date_compound: self.date_compound,
			description: description.into(),
		}
	}
}
