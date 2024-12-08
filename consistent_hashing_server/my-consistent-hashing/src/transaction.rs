use std::fmt::{self};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Transaction<T: fmt::Display + PartialEq, N: fmt::Display + PartialOrd> {
    pub source: T,
    pub destination: T,
    pub min: N,
    pub max: N,
    exception: bool
}

impl<T: fmt::Display + PartialEq, N: fmt::Display + PartialOrd> Transaction<T, N> {
    pub fn new(source: T, destination: T, min: N, max: N) -> Self {
        Self {
            exception: (min > max),
            source,
            destination,
            min,
            max
        }
    }
    pub fn in_range(&self, num: N) -> bool {
        if !self.exception {
            return num > self.min && num <= self.max;
        }
        return num <= self.max || num > self.min;
    }
}

impl<T: fmt::Display + PartialEq, N: fmt::Display + PartialOrd> fmt::Display for Transaction<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringy = match self.exception {
            true => {
                format!("[0,{}] u ]{},..[", self.max, self.min)
            },
            _ => {
                format!("]{},{}]", self.min, self.max)
            }
        };
        write!(f, "source: {}  -  destination: {}  -  range: {}", self.source, self.destination, stringy)
    }
}

impl<T: fmt::Display + PartialEq, N: fmt::Display + PartialOrd> PartialEq for Transaction<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.destination == other.destination
            && self.exception == other.exception
            && self.min == other.min
            && self.max == other.max
    }
}