// Copyright (c) 2017-2021 Fabian Schuiki

//! Primary and secondary tables.
//!
//! This module implements primary tables which are used to associate some data
//! with a dense, opaque, integer id; and secondary tables which are used to
//! associate additional data with the primary table.

use hibitset::{BitSet, BitSetLike};
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

/// An opaque key to uniquely identify a table entry.
pub trait TableKey: Copy {
    /// Create a new table key from an index.
    fn new(index: usize) -> Self;

    /// Create an invalid table key.
    fn invalid() -> Self;

    /// Return the index wrapped within this table key.
    fn index(self) -> usize;

    /// Return whether this table key is invalid.
    fn is_invalid(self) -> bool;
}

/// Generate a new opaque table key struct.
#[macro_export]
macro_rules! impl_table_key {
    ($($(#[$m:meta])* struct $name:ident($ity:ty) as $display_prefix:expr;)*) => {
        $(
            $(#[$m])*
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

                fn invalid() -> Self {
                    $name(<$ity>::max_value())
                }

                fn index(self) -> usize {
                    self.0 as usize
                }

                fn is_invalid(self) -> bool {
                    self.0 == <$ity>::max_value()
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
#[derive(Clone, Serialize, Deserialize)]
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

    /// Return an iterator over the keys and mutable values in the table.
    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (I, &'a mut V)> + 'a {
        self.storage.iter_mut().map(|(&k, v)| (I::new(k), v))
    }

    /// Return an iterator over the keys in the table.
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = I> + 'a {
        self.storage.keys().cloned().map(I::new)
    }

    /// Return an iterator over the values in the table.
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a {
        self.storage.values()
    }

    /// Return an iterator over the mutable values in the table.
    pub fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a {
        self.storage.values_mut()
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
#[derive(Clone, Serialize, Deserialize)]
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

/// A primary table that provides dense key-based storage.
#[derive(Clone)]
pub struct PrimaryTable2<I, V> {
    storage: Vec<V>,
    count: usize,
    used: BitSet,
    free: BitSet,
    unused: PhantomData<I>,
}

impl<I, V> PrimaryTable2<I, V> {
    /// Create a new primary table.
    pub fn new() -> Self {
        Self {
            storage: Default::default(),
            count: 0,
            used: Default::default(),
            free: Default::default(),
            unused: PhantomData,
        }
    }
}

impl<I, V> Default for PrimaryTable2<I, V> {
    fn default() -> PrimaryTable2<I, V> {
        PrimaryTable2::new()
    }
}

impl<I: TableKey, V: Default> PrimaryTable2<I, V> {
    /// Add a new entry to the table.
    ///
    /// Returns the key under which the entry can be accessed again.
    pub fn add(&mut self, value: V) -> I {
        self.count += 1;
        if let Some(id) = (&self.free).iter().next() {
            self.used.add(id);
            self.free.remove(id);
            self.storage[id as usize] = value;
            return I::new(id as usize);
        }
        let id = self.storage.len() as u32;
        self.used.add(id);
        self.storage.push(value);
        I::new(id as usize)
    }

    /// Remove an entry from the table.
    ///
    /// Panics if the entry does not exist.
    pub fn remove(&mut self, key: I) -> V {
        let id = key.index() as u32;
        assert!(self.used.contains(id));
        self.count -= 1;
        self.used.remove(id);
        self.free.add(id);
        std::mem::replace(&mut self.storage[id as usize], Default::default())
    }

    /// Get the number of entries for which storage is allocated.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Return an iterator over the keys and values in the table.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (I, &'a V)> + 'a {
        (&self.used)
            .iter()
            .map(move |i| (I::new(i as usize), &self.storage[i as usize]))
    }

    /// Return an iterator over the keys in the table.
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = I> + 'a {
        (&self.used).iter().map(|i| I::new(i as usize))
    }

    /// Return an iterator over the values in the table.
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> + 'a {
        (&self.used).iter().map(move |i| &self.storage[i as usize])
    }

    /// Return an iterator over the mutable values in the table.
    pub fn values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut V> + 'a {
        let slf: &'a _ = &self.used;
        let stor: &'a _ = &self.storage;
        slf.iter().map(move |i| {
            let v = &stor[i as usize] as *const V;
            let v = v as *mut V;
            // This is safe because the mutable borrow on self is alive for as
            // long as the iterator is alive ('a).
            let v = unsafe { v.as_mut().unwrap() };
            v
        })
    }
}

impl<I: TableKey, V> Index<I> for PrimaryTable2<I, V> {
    type Output = V;

    fn index(&self, idx: I) -> &V {
        &self.storage[idx.index()]
    }
}

impl<I: TableKey, V> IndexMut<I> for PrimaryTable2<I, V> {
    fn index_mut(&mut self, idx: I) -> &mut V {
        &mut self.storage[idx.index()]
    }
}

impl<I, V> Serialize for PrimaryTable2<I, V>
where
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.count))?;
        for i in &self.used {
            map.serialize_entry(&i, &self.storage[i as usize])?;
        }
        map.end()
    }
}

impl<'de, I, V> Deserialize<'de> for PrimaryTable2<I, V>
where
    V: Deserialize<'de> + Default + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<I, V>(PhantomData<fn() -> PrimaryTable2<I, V>>);

        impl<'de, I, V> serde::de::Visitor<'de> for Visitor<I, V>
        where
            V: Deserialize<'de> + Default + Clone,
        {
            type Value = PrimaryTable2<I, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut map = PrimaryTable2::default();
                while let Some((key, value)) = access.next_entry()? {
                    let key: usize = key; // type hint
                    if key >= map.storage.len() {
                        map.storage.resize(key + 1, Default::default());
                    }
                    map.storage[key] = value;
                    map.used.add(key as u32);
                    map.count += 1;
                }
                for i in 0..map.storage.len() {
                    if !map.used.contains(i as u32) {
                        map.free.add(i as u32);
                    }
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(Visitor(PhantomData))
    }
}
