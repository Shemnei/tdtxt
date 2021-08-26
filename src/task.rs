use std::fmt;

use crate::date::DateCompound;
use crate::description::Description;
use crate::parse::{Parse, Parser};
use crate::priority::Priority;
use crate::state::State;

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
