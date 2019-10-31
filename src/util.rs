// Copyright (c) 2017-2019 Fabian Schuiki

//! Various utility functions that fit nowhere else.

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
