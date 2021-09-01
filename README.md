# tdtxt

[![Build](https://img.shields.io/github/workflow/status/Shemnei/tdtxt/CI/main?logo=github&style=for-the-badge)](https://github.com/Shemnei/tdtxt/actions?query=branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/tdtxt?logo=rust&style=for-the-badge)](https://crates.io/crates/tdtxt)
[![Documentation](https://img.shields.io/badge/docs.rs-tdtxt-blue?logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K&style=for-the-badge)](https://docs.rs/tdtxt)
[![GitHub Issues](https://img.shields.io/github/issues/Shemnei/tdtxt?logo=github&style=for-the-badge)](https://github.com/Shemnei/tdtxt/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc)
[![Unlicense License](https://img.shields.io/crates/l/tdtxt?style=for-the-badge)](https://unlicense.org/)

A rust library for de(serializing) files and text in the [todo.txt format](https://github.com/todotxt/todo.txt).

## STATE

**This crate is still in development, the api may not stable and could change in the future.**

This crate should not be used in production code.

## Usage

Add it to the dependencies of your `Cargo.toml`:

```toml
[dependencies]
tdtxt = "0.1"
```

Then use it:

```rust
use std::str::FromStr as _;

use tdtxt::{Task, Date, State, Priority, DateCompound};

let line = "x (A) 2016-05-20 2016-04-30 measure space for +chapelShelving @chapel due:2016-05-30";
let task = Task::from_str(line).unwrap();

assert_eq!(task.state(), &State::Done);
assert_eq!(task.priority(), Some(&Priority::A));
assert_eq!(task.date_compound(), Some(&DateCompound::Completed { created: Date::ymd(2016, 4, 30), completed: Date::ymd(2016, 5, 20) }));
assert_eq!(task.description().description(), "measure space for +chapelShelving @chapel due:2016-05-30");
assert_eq!(task.description().projects().collect::<Vec<_>>(), vec!["chapelShelving"]);
assert_eq!(task.description().contexts().collect::<Vec<_>>(), vec!["chapel"]);
assert_eq!(task.description().custom().collect::<Vec<_>>(), vec![("due", "2016-05-30")]);
```

```rust
use std::str::FromStr as _;

use tdtxt::{Task, Date, State, Priority, DateCompound};

let line = "x (A) 2016-05-20 2016-04-30 measure space for +chapelShelving @chapel due:2016-05-30";
let task = Task::build()
    .state(State::Done)
    .priority(Priority::A)
    .date_compound(DateCompound::completed(Date::ymd(2016, 4, 30), Date::ymd(2016, 5, 20)))
    .build("measure space for +chapelShelving @chapel due:2016-05-30");

assert_eq!(format!("{}", task), line);

assert_eq!(task.state(), &State::Done);
assert_eq!(task.priority(), Some(&Priority::A));
assert_eq!(task.date_compound(), Some(&DateCompound::Completed { created: Date::ymd(2016, 4, 30), completed: Date::ymd(2016, 5, 20) }));
assert_eq!(task.description().description(), "measure space for +chapelShelving @chapel due:2016-05-30");
assert_eq!(task.description().projects().collect::<Vec<_>>(), vec!["chapelShelving"]);
assert_eq!(task.description().contexts().collect::<Vec<_>>(), vec!["chapel"]);
assert_eq!(task.description().custom().collect::<Vec<_>>(), vec![("due", "2016-05-30")]);
```

## Features

### Serde (`serde`)

Serialize and deserialize the Task struct with serde.

#### Examples

```rust
use tdtxt::{Task, Date, State, Priority, DateCompound};

let task_should = Task::build()
    .state(State::Done)
    .priority(Priority::A)
    .date_compound(DateCompound::completed(
        Date::ymd(2016, 4, 30),
        Date::ymd(2016, 5, 20),
    ))
    .build("measure space for +chapelShelving @chapel due:2016-05-30");

let json = serde_json::to_string_pretty(&task_should).unwrap();
println!("{}", &json);
```

Example json output:

```json
{
  "state": "Done",
  "priority": "A",
  "created": "2016-04-30",
  "completed": "2016-05-20",
  "description": "measure space for +chapelShelving @chapel due:2016-05-30"
}
```

**NOTE**

The order in which `created` and `completed` appear matters.
