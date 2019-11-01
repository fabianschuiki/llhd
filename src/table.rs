// Copyright (c) 2017-2019 Fabian Schuiki

//! Primary and secondary tables.
//!
//! This module implements primary tables which are used to associate some data
//! with a dense, opaque, integer id; and secondary tables which are used to
//! associate additional data with the primary table.

use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

/// An opaque key to uniquely identify a table entry.
pub trait TableKey: Copy {
    /// Create a new table key from an index.
    fn new(index: usize) -> Self;
    /// Return the index wrapped within this table key.
    fn index(self) -> usize;
}

/// Generate a new opaque table key struct.
#[macro_export]
macro_rules! impl_table_key {
    ($($(#[$m:meta])* struct $name:ident($ity:ty) as $display_prefix:expr;)*) => {
        $(
            $(#[$m])*
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name($ity);

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}{}", $display_prefix, self.0)
                }
            }

            impl std::fmt::Debug for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", self)
                }
            }

            impl $crate::table::TableKey for $name {
                fn new(index: usize) -> Self {
                    $name(index as $ity)
                }

                fn index(self) -> usize {
                    self.0 as usize
                }
            }
        )*
    };
}

/// Generate the `Index` and `IndexMut` operations for a contained table.
#[macro_export]
macro_rules! impl_table_indexing {
    ($target:path, $($field:ident).+, $key:ty, $value:ty) => {
        impl std::ops::Index<$key> for $target {
            type Output = $value;

            fn index(&self, idx: $key) -> &$value {
                &self.$($field).*[idx]
            }
        }

        impl std::ops::IndexMut<$key> for $target {
            fn index_mut(&mut self, idx: $key) -> &mut $value {
                &mut self.$($field).*[idx]
            }
        }
    };
}

/// A primary table that provides dense key-based storage.
#[derive(Clone)]
pub struct PrimaryTable<I, V> {
    next: usize,
    pub(crate) storage: HashMap<usize, V>,
    unused: PhantomData<I>,
}

impl<I, V> PrimaryTable<I, V> {
    /// Create a new primary table.
    pub fn new() -> Self {
        Self {
            next: 0,
            storage: Default::default(),
            unused: PhantomData,
        }
    }
}

impl<I, V> Default for PrimaryTable<I, V> {
    fn default() -> PrimaryTable<I, V> {
        PrimaryTable::new()
    }
}

impl<I: TableKey, V> PrimaryTable<I, V> {
    /// Add a new entry to the table.
    ///
    /// Returns the key under which the entry can be accessed again.
    pub fn add(&mut self, value: V) -> I {
        let index = self.next;
        self.next += 1;
        self.storage.insert(index, value);
        I::new(index)
    }

    /// Remove an entry from the table.
    ///
    /// Panics if the entry does not exist.
    pub fn remove(&mut self, key: I) {
        self.storage.remove(&key.index()).expect("key not in table");
    }

    /// Return an iterator over the keys and values in the table.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (I, &'a V)> + 'a {
        self.storage.iter().map(|(&k, v)| (I::new(k), v))
    }

    /// Return an iterator over the keys in the table.
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = I> + 'a {
        self.storage.keys().cloned().map(I::new)
    }

    /// Return an iterator over the values in the table.
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a {
        self.storage.values()
    }
}

impl<I: TableKey, V> Index<I> for PrimaryTable<I, V> {
    type Output = V;

    fn index(&self, idx: I) -> &V {
        self.storage.get(&idx.index()).expect("key not in table")
    }
}

impl<I: TableKey, V> IndexMut<I> for PrimaryTable<I, V> {
    fn index_mut(&mut self, idx: I) -> &mut V {
        self.storage
            .get_mut(&idx.index())
            .expect("key not in table")
    }
}

/// A secondary table that associates additional information with entries in a
/// primary table.
#[derive(Clone)]
pub struct SecondaryTable<I, V> {
    pub(crate) storage: HashMap<usize, V>,
    unused: PhantomData<I>,
}

impl<I, V> SecondaryTable<I, V> {
    /// Create a new empty table.
    pub fn new() -> Self {
        Self {
            storage: Default::default(),
            unused: PhantomData,
        }
    }
}

impl<I: TableKey, V> SecondaryTable<I, V> {
    /// Add an entry to the table.
    ///
    /// The user must provide the key with which the information is associated.
    pub fn add(&mut self, key: I, value: V) {
        if self.storage.insert(key.index(), value).is_some() {
            panic!("key already in table");
        }
    }

    /// Remove an entry from the table.
    pub fn remove(&mut self, key: I) -> Option<V> {
        self.storage.remove(&key.index())
    }

    /// Check whether an entry exists in the table.
    pub fn contains(&self, key: I) -> bool {
        self.storage.contains_key(&key.index())
    }

    /// Get an entry from the table, if one exists.
    pub fn get(&self, key: I) -> Option<&V> {
        self.storage.get(&key.index())
    }

    /// Get a mutable entry from the table, if one exists.
    pub fn get_mut(&mut self, key: I) -> Option<&mut V> {
        self.storage.get_mut(&key.index())
    }
}

impl<I, V> Default for SecondaryTable<I, V> {
    fn default() -> SecondaryTable<I, V> {
        SecondaryTable::new()
    }
}

impl<I: TableKey, V> Index<I> for SecondaryTable<I, V> {
    type Output = V;

    fn index(&self, idx: I) -> &V {
        self.storage
            .get(&idx.index())
            .expect("key not in secondary table")
    }
}

impl<I: TableKey, V> IndexMut<I> for SecondaryTable<I, V> {
    fn index_mut(&mut self, idx: I) -> &mut V {
        self.storage
            .get_mut(&idx.index())
            .expect("key not in secondary table")
    }
}
