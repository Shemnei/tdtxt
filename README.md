# tdtxt

[![Continuous Integration](https://github.com/Shemnei/tdtxt/workflows/CI/badge.svg)](https://github.com/Shemnei/tdtxt/actions)
[![Crates.io](https://img.shields.io/crates/v/tdtxt)](https://crates.io/crates/tdtxt)
[![Documentation](https://docs.rs/tdtxt/badge.svg)](https://docs.rs/tdtxt)
[![GitHub Issues](https://img.shields.io/github/issues/Shemnei/tdtxt)](https://github.com/Shemnei/tdtxt/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc)
[![Unlicense License](https://img.shields.io/crates/l/tdtxt)](https://unlicense.org/)

A rust library for de(serializing) files and text in the [todo.txt format](https://github.com/todotxt/todo.txt).

## STATE

**This crate is still early in development, the api is not stable and will change in the future.**

Having said that, the core of the crate (parsing) is implemented and should function correctly.

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
