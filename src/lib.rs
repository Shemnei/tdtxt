//! # Examples
//!
//! ```rust
//! use std::str::FromStr as _;
//!
//! use tdtxt::{Task, Date, State, Priority, DateCompound};
//!
//! let line = "x (A) 2016-05-20 2016-04-30 measure space for +chapelShelving @chapel due:2016-05-30";
//! let task = Task::from_str(line).unwrap();
//!
//! assert_eq!(task.state(), &State::Done);
//! assert_eq!(task.priority(), Some(&Priority::A));
//! assert_eq!(task.date_compound(), Some(&DateCompound::Completed { created: Date::ymd(2016, 4, 30), completed: Date::ymd(2016, 5, 20) }));
//! assert_eq!(task.description().description(), "measure space for +chapelShelving @chapel due:2016-05-30");
//! assert_eq!(task.description().projects().collect::<Vec<_>>(), vec!["chapelShelving"]);
//! assert_eq!(task.description().contexts().collect::<Vec<_>>(), vec!["chapel"]);
//! assert_eq!(task.description().custom().collect::<Vec<_>>(), vec![("due", "2016-05-30")]);
//! ```
//!
//! ```rust
//! use std::str::FromStr as _;
//!
//! use tdtxt::{Task, Date, State, Priority, DateCompound};
//!
//! let line = "x (A) 2016-05-20 2016-04-30 measure space for +chapelShelving @chapel due:2016-05-30";
//! let task = Task::build()
//!     .state(State::Done)
//!     .priority(Priority::A)
//!     .date_compound(DateCompound::completed(Date::ymd(2016, 4, 30), Date::ymd(2016, 5, 20)))
//!     .build("measure space for +chapelShelving @chapel due:2016-05-30");
//!
//! assert_eq!(format!("{}", task), line);
//!
//! assert_eq!(task.state(), &State::Done);
//! assert_eq!(task.priority(), Some(&Priority::A));
//! assert_eq!(task.date_compound(), Some(&DateCompound::Completed { created: Date::ymd(2016, 4, 30), completed: Date::ymd(2016, 5, 20) }));
//! assert_eq!(task.description().description(), "measure space for +chapelShelving @chapel due:2016-05-30");
//! assert_eq!(task.description().projects().collect::<Vec<_>>(), vec!["chapelShelving"]);
//! assert_eq!(task.description().contexts().collect::<Vec<_>>(), vec!["chapel"]);
//! assert_eq!(task.description().custom().collect::<Vec<_>>(), vec![("due", "2016-05-30")]);
//! ```

#![allow(dead_code, rustdoc::private_intra_doc_links)]
#![deny(
    // Documentation
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	missing_docs,
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

mod date;
mod description;
mod priority;
mod state;
mod task;

mod parse;
mod span;

pub use crate::date::{Date, DateCompound};
pub use crate::description::Description;
pub use crate::priority::Priority;
pub use crate::state::State;
pub use crate::task::{Task, TaskBuilder};

pub mod prelude {
	//! The prelude exports all components needed for regular use.
	//!
	//! # Examples
	//!
	//! ```rust
	//! use tdtxt::prelude::*;
	//! ```

	pub use crate::date::{Date, DateCompound};
	pub use crate::description::Description;
	pub use crate::priority::Priority;
	pub use crate::state::State;
	pub use crate::task::{Task, TaskBuilder};
}

#[cfg(test)]
mod tests {
	use pretty_assertions::assert_eq;

	use crate::date::{Date, DateCompound};
	use crate::description::Description;
	use crate::parse::*;
	use crate::priority::Priority;
	use crate::state::State;
	use crate::task::{ParseTaskError, Task};

	#[test]
	fn task_display() {
		let task = Task {
			state: State::Done,
			priority: Some(Priority::H),
			date_compound: None,
			description: Description::new("Hello World"),
		};

		assert_eq!(task.to_string(), "x (H) Hello World");
	}

	#[test]
	fn task_parse() {
		let input = b"x";
		let mut parser = Parser::new(input);

		assert_eq!(State::parse(&mut parser), Ok(State::Done));

		let input = b"(H)";
		let mut parser = Parser::new(input);

		assert_eq!(Priority::parse(&mut parser), Ok(Priority::H));

		let input = b"2020-01-01";
		let mut parser = Parser::new(input);

		assert_eq!(Date::parse(&mut parser), Ok(Date::ymd(2020, 01, 01)));

		let input = b"1234-07-16";
		let mut parser = Parser::new(input);

		let d = DateCompound::Created { created: Date::ymd(1234, 07, 16) };
		assert_eq!(DateCompound::parse(&mut parser), Ok(d));

		let input = b"2000-01-01 1970-01-01";
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

		let task = Task {
			state: State::Done,
			priority: Some(Priority::Z),
			date_compound: Some(DateCompound::Created {
				created: Date::ymd(2020, 01, 01),
			}),
			description: Description::new("Hello World"),
		};

		assert_eq!(Task::parse(&mut parser), Ok(task));
	}

	#[test]
	fn task_example() {
		let input = b"(A) Thank Mom for the meatballs @phone
(B) Schedule Goodwill pickup +GarageSale @phone
Post signs around the neighborhood +GarageSale
@GroceryStore Eskimo pies";
		let mut parser = Parser::new(input);

		let task = Task::build()
			.priority(Priority::A)
			.build("Thank Mom for the meatballs @phone");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		let task = Task::build()
			.priority(Priority::B)
			.build("Schedule Goodwill pickup +GarageSale @phone");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		let task = Task::build()
			.build("Post signs around the neighborhood +GarageSale");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		let task = Task::build().build("@GroceryStore Eskimo pies");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	#[test]
	fn task_rule1() {
		let input = b"(A) Call Mom";
		let mut parser = Parser::new(input);

		let task = Task::build().priority(Priority::A).build("Call Mom");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));

		let input = b"Really gotta call Mom (A) @phone @someday
(b) Get back to the boss
(B)->Submit TPS report";
		let mut parser = Parser::new(input);

		let task =
			Task::build().build("Really gotta call Mom (A) @phone @someday");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		let task = Task::build().build("(b) Get back to the boss");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		let task = Task::build().build("(B)->Submit TPS report");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	#[test]
	fn task_rule2() {
		let input = b"2011-03-02 Document +TodoTxt task format
(A) 2011-03-02 Call Mom";
		let mut parser = Parser::new(input);

		let task = Task::build()
			.date_compound(DateCompound::Created {
				created: Date::ymd(2011, 03, 02),
			})
			.build("Document +TodoTxt task format");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		let task = Task::build()
			.priority(Priority::A)
			.date_compound(DateCompound::Created {
				created: Date::ymd(2011, 03, 02),
			})
			.build("Call Mom");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));

		let input = b"(A) Call Mom 2011-03-02";
		let mut parser = Parser::new(input);

		let task =
			Task::build().priority(Priority::A).build("Call Mom 2011-03-02");
		assert_eq!(Task::parse(&mut parser), Ok(task));

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	#[test]
	fn task_rule3() {
		let input =
			b"(A) Call Mom +Family +PeaceLoveAndHappiness @iphone @phone";
		let mut parser = Parser::new(input);

		let task_should = Task::build()
			.priority(Priority::A)
			.build("Call Mom +Family +PeaceLoveAndHappiness @iphone @phone");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		let projects_should = vec!["Family", "PeaceLoveAndHappiness"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should = vec!["iphone", "phone"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));

		let input = b"Email SoAndSo at soandso@example.com";
		let mut parser = Parser::new(input);

		let task_should =
			Task::build().build("Email SoAndSo at soandso@example.com");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));

		let input = b"Learn how to add 2+2";
		let mut parser = Parser::new(input);

		let task_should = Task::build().build("Learn how to add 2+2");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	#[test]
	fn task_parse_full() {
		let input =
            b"x (J) 1990-01-01 1980-01-01 Wait ten year @home for +century_waiting author:me";
		let mut parser = Parser::new(input);

		let task_should = Task::build()
			.state(State::Done)
			.priority(Priority::J)
			.date_compound(DateCompound::Completed {
				created: Date::ymd(1980, 01, 01),
				completed: Date::ymd(1990, 01, 01),
			})
			.build("Wait ten year @home for +century_waiting author:me");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(task_is.to_string().as_bytes(), input);
		let projects_should: Vec<&str> = vec!["century_waiting"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["home"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![("author", "me")];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	#[test]
	fn task_parse_edge() {
		let input =
			b"add + some more not::valid also:not:valid @home @work should:";
		let mut parser = Parser::new(input);

		let task_should = Task::build().build(
			"add + some more not::valid also:not:valid @home @work should:",
		);
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["home", "work"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));

		let input = b"2014-10 key:value";
		let mut parser = Parser::new(input);

		let task_should = Task::build().build("2014-10 key:value");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![("key", "value")];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));

		let input = b"x  How:you doin (A)";
		let mut parser = Parser::new(input);

		let task_should =
			Task::build().state(State::Done).build(" How:you doin (A)");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![("How", "you")];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	// http://todotxt.org/todo.txt
	#[test]
	fn task_parse_example() {
		let input = b"(A) Call Mom @Phone +Family
(A) Schedule annual checkup +Health
(B) Outline chapter 5 +Novel @Computer
(C) Add cover sheets @Office +TPSReports
Plan backyard herb garden @Home
Pick up milk @GroceryStore
Research self-publishing services +Novel @Computer
x Download Todo.txt mobile app @Phone";
		let mut parser = Parser::new(input);

		let task_should = Task::build()
			.priority(Priority::A)
			.build("Call Mom @Phone +Family");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"(A) Call Mom @Phone +Family"
		);
		let projects_should: Vec<&str> = vec!["Family"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["Phone"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should = Task::build()
			.priority(Priority::A)
			.build("Schedule annual checkup +Health");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"(A) Schedule annual checkup +Health"
		);
		let projects_should: Vec<&str> = vec!["Health"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should = Task::build()
			.priority(Priority::B)
			.build("Outline chapter 5 +Novel @Computer");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"(B) Outline chapter 5 +Novel @Computer"
		);
		let projects_should: Vec<&str> = vec!["Novel"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["Computer"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should = Task::build()
			.priority(Priority::C)
			.build("Add cover sheets @Office +TPSReports");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"(C) Add cover sheets @Office +TPSReports"
		);
		let projects_should: Vec<&str> = vec!["TPSReports"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["Office"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should =
			Task::build().build("Plan backyard herb garden @Home");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"Plan backyard herb garden @Home"
		);
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["Home"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should = Task::build().build("Pick up milk @GroceryStore");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"Pick up milk @GroceryStore"
		);
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["GroceryStore"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should = Task::build()
			.build("Research self-publishing services +Novel @Computer");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"Research self-publishing services +Novel @Computer"
		);
		let projects_should: Vec<&str> = vec!["Novel"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["Computer"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		let task_should = Task::build()
			.state(State::Done)
			.build("Download Todo.txt mobile app @Phone");
		let task_is = Task::parse(&mut parser);
		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"x Download Todo.txt mobile app @Phone"
		);
		let projects_should: Vec<&str> = vec![];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["Phone"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(Task::parse(&mut parser), Err(ParseTaskError));
	}

	#[test]
	fn parse_git_example() {
		use std::str::FromStr;

		let input = "x (A) 2016-05-20 2016-04-30 measure space for \
		             +chapelShelving @chapel due:2016-05-30";

		let task_is = Task::from_str(input);

		let task_should = Task::build()
			.state(State::Done)
			.priority(Priority::A)
			.date_compound(DateCompound::completed(
				Date::ymd(2016, 4, 30),
				Date::ymd(2016, 5, 20),
			))
			.build("measure space for +chapelShelving @chapel due:2016-05-30");

		assert_eq!(task_is, Ok(task_should));
		let task_is = task_is.unwrap();
		assert_eq!(
			task_is.to_string().as_bytes(),
			b"x (A) 2016-05-20 2016-04-30 measure space for \
		             +chapelShelving @chapel due:2016-05-30"
		);
		let projects_should: Vec<&str> = vec!["chapelShelving"];
		assert_eq!(
			task_is.description.projects().collect::<Vec<_>>(),
			projects_should
		);
		let contexts_should: Vec<&str> = vec!["chapel"];
		assert_eq!(
			task_is.description.contexts().collect::<Vec<_>>(),
			contexts_should
		);
		let custom_should: Vec<(&str, &str)> = vec![("due", "2016-05-30")];
		assert_eq!(
			task_is.description.custom().collect::<Vec<_>>(),
			custom_should
		);

		assert_eq!(
			Date::from_str(task_is.description().custom().next().unwrap().1)
				.unwrap(),
			Date::ymd_opt(2016, 5, 30).unwrap()
		);
	}

	#[test]
	fn priority_ord() {
		assert!(Priority::A > Priority::B);
		assert!(Priority::A == Priority::A);
		assert!(Priority::Z < Priority::A);
	}
}
