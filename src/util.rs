// Copyright (c) 2017-2019 Fabian Schuiki

//! Various utility functions that fit nowhere else.

use std;
use std::ops::Index;

/// Formats a slice of elements that implement the `std::fmt::Display` trait as
/// a list with the given separator.
pub(crate) fn write_implode<T, I, S>(f: &mut std::fmt::Formatter, sep: S, it: I) -> std::fmt::Result
where
    T: std::fmt::Display,
    I: std::iter::Iterator<Item = T>,
    S: std::fmt::Display,
{
    write_implode_with(f, sep, it, |f, x| x.fmt(f))
}

/// Formats a slice of elements using a callback function, inserting the given
/// separator between consecutive elments.
pub(crate) fn write_implode_with<T, I, S, F>(
    f: &mut std::fmt::Formatter,
    sep: S,
    mut it: I,
    write: F,
) -> std::fmt::Result
where
    I: std::iter::Iterator<Item = T>,
    S: std::fmt::Display,
    F: Fn(&mut std::fmt::Formatter, T) -> std::fmt::Result,
{
    if let Some(x) = it.next() {
        write(f, x)?;
    }
    for x in it {
        write!(f, "{}", sep)?;
        write(f, x)?;
    }
    Ok(())
}

/// An iterator over a sequence of keys into a map. The iterator produces the
/// items from the map, in the order in which their keys appear in the slice.
pub struct IndirectMapIter<'tf, T: Iterator + 'tf, V: 'tf> {
    refs: T,
    map: &'tf Index<T::Item, Output = V>,
}

impl<'tf, T: Iterator + 'tf, V> IndirectMapIter<'tf, T, V> {
    pub fn new(refs: T, map: &'tf Index<T::Item, Output = V>) -> IndirectMapIter<'tf, T, V> {
        IndirectMapIter {
            refs: refs,
            map: map,
        }
    }
}

impl<'tf, T: Iterator + 'tf, V> Iterator for IndirectMapIter<'tf, T, V> {
    type Item = &'tf V;

    fn next(&mut self) -> Option<&'tf V> {
        self.refs.next().map(|r| &self.map[r])
    }
}
