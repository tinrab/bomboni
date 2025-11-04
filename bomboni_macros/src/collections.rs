//! Collection macros.

/// A macro that counts the number of times a pattern is repeated.
///
/// # Examples
///
/// ```
/// use bomboni_macros::count_repeating;
///
/// let count = count_repeating!(1, 1, 2, 3, 5);
/// assert_eq!(count, 5);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! count_repeating {
    (@single $($x:tt)*) => (());
    ($($rest:expr),*) => (<[()]>::len(&[$(count_repeating!(@single $rest)),*]));
}

/// A macro that creates a new `BTreeMap` instance with the given key-value pairs.
///
/// # Examples
///
/// Create a map of key-value pairs.
///
/// ```
/// use bomboni_macros::btree_map;
///
/// let map = btree_map! {
///     1 => "first",
///     2 => "second",
/// };
/// assert_eq!(map.get(&1), Some(&"first"));
/// assert_eq!(map.get(&2), Some(&"second"));
/// ```
#[macro_export(local_inner_macros)]
macro_rules! btree_map {
    () => {
        ::std::collections::BTreeMap::new()
    };
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut _map = btree_map!();
        $(
            _map.insert($key, $value);
        )*
        _map
    }};
}

/// A macro that creates a new `BTreeMap` instance with the given key-value pairs.
/// The same as `btree_map!`, but converts keys and values to the target type.
///
/// # Examples
///
/// Create a map of key-value pairs.
///
/// ```
/// # use std::collections::BTreeMap;
/// use bomboni_macros::btree_map_into;
///
/// let map: BTreeMap<i32, String> = btree_map_into! {
///     1 => "first",
///     2 => "second",
/// };
/// assert_eq!(map.get(&1), Some(&"first".to_string()));
/// assert_eq!(map.get(&2), Some(&"second".to_string()));
/// ```
#[macro_export(local_inner_macros)]
macro_rules! btree_map_into {
    ($($key:expr => $value:expr),* $(,)?) => {
        btree_map!($($key.into() => $value.into()),*)
    };
}

/// A macro that creates a new `HashMap` instance with the given key-value pairs.
///
/// # Examples
///
/// Create a map of key-value pairs.
///
/// ```
/// use bomboni_macros::hash_map;
///
/// let map = hash_map! {
///     1 => "first",
///     2 => "second",
/// };
/// assert_eq!(map.get(&1), Some(&"first"));
/// assert_eq!(map.get(&2), Some(&"second"));
/// ```
///
/// Create a map with a given capacity.
///
/// ```
/// use bomboni_macros::hash_map;
/// # use std::collections::HashMap;
///
/// let _map: HashMap<i32, String> = hash_map!(100);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! hash_map {
    () => {
        ::std::collections::HashMap::new()
    };
    ($capacity:expr) => {
        ::std::collections::HashMap::with_capacity($capacity)
    };
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut _map = hash_map!(count_repeating!($($key),*));
        $(
            _map.insert($key, $value);
        )*
        _map
    }};
}
/// A macro that creates a new `HashMap` instance with the given key-value pairs.
/// The same as `hash_map!`, but converts keys and values to the target type.
///
/// # Examples
///
/// Create a map of key-value pairs.
///
/// ```
/// # use std::collections::HashMap;
/// use bomboni_macros::hash_map_into;
///
/// let map: HashMap<i32, String> = hash_map_into! {
///     1 => "first",
///     2 => "second",
/// };
/// assert_eq!(map.get(&1), Some(&"first".to_string()));
/// assert_eq!(map.get(&2), Some(&"second".to_string()));
/// ```
#[macro_export(local_inner_macros)]
macro_rules! hash_map_into {
    ($($key:expr => $value:expr),* $(,)?) => {
        hash_map!($($key.into() => $value.into()),*)
    };
}

/// A macro that creates a new `BTreeSet` and inserts the given values into it.
///
/// # Examples
///
/// ```
/// use bomboni_macros::btree_set;
///
/// let set = btree_set![1, 2, 3];
/// assert!(set.contains(&1));
/// assert!(set.contains(&2));
/// assert_eq!(set.len(), 3);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! btree_set {
    () => {
        ::std::collections::BTreeSet::new()
    };
    ($($value:expr),* $(,)?) => {{
        let mut _set = btree_set!();
        $(
            _set.insert($value);
        )*
        _set
    }};
}

/// A macro that creates a new `BTreeSet` and inserts the given values into it.
/// The same as `btree_set!`, but converts values to the target type.
///
/// # Examples
///
/// ```
/// # use std::collections::BTreeSet;
/// use bomboni_macros::btree_set_into;
///
/// let set: BTreeSet<i32> = btree_set_into![1, 2, 3];
/// assert!(set.contains(&1));
/// assert!(set.contains(&2));
/// assert_eq!(set.len(), 3);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! btree_set_into {
    ($($value:expr),* $(,)?) => {
        btree_set!($($value.into()),*)
    };
}

/// A macro that creates a new `HashSet` and inserts the given values into it.
///
/// # Examples
///
/// ```
/// use bomboni_macros::hash_set;
///
/// let set = hash_set![1, 2, 3];
/// assert!(set.contains(&1));
/// assert!(set.contains(&2));
/// assert_eq!(set.len(), 3);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! hash_set {
    () => {
        ::std::collections::HashSet::new()
    };
    ($($value:expr),* $(,)?) => {{
        let mut _set = hash_set!();
        $(
            _set.insert($value);
        )*
        _set
    }};
}

/// A macro that creates a new `HashSet` and inserts the given values into it.
/// The same as `hash_set!`, but converts values to the target type.
///
/// # Examples
///
/// ```
/// # use std::collections::HashSet;
/// use bomboni_macros::hash_set_into;
///
/// let set: HashSet<i32> = hash_set_into![1, 2, 3];
/// assert!(set.contains(&1));
/// assert!(set.contains(&2));
/// assert_eq!(set.len(), 3);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! hash_set_into {
    ($($value:expr),* $(,)?) => {
        hash_set!($($value.into()),*)
    };
}

/// A macro that creates a new `VecDeque` instance with the given values.
///
/// # Examples
///
/// ```
/// # use std::collections::VecDeque;
/// use bomboni_macros::vec_deque;
///
/// let deque = vec_deque![1, 2, 3];
/// assert_eq!(deque, VecDeque::from(vec![1, 2, 3]));
/// ```
#[macro_export(local_inner_macros)]
macro_rules! vec_deque {
    () => {
        ::std::collections::VecDeque::new()
    };
    ($elem:expr; $n:expr) => { ::std::vec![$elem; $n] };
    ($($value:expr),* $(,)?) => {{
        ::std::collections::VecDeque::from([
            $($value),*
        ])
    }};
}

/// A macro that creates a new `VecDeque` instance with the given values.
/// The same as `vec_deque!`, but converts values to the target type.
///
/// # Examples
///
/// ```
/// # use std::collections::VecDeque;
/// use bomboni_macros::vec_deque_into;
///
/// let deque: VecDeque<i32> = vec_deque_into![1, 2, 3];
/// assert_eq!(deque, VecDeque::from(vec![1, 2, 3]));
/// ```
#[macro_export(local_inner_macros)]
macro_rules! vec_deque_into {
    ($($value:expr),* $(,)?) => {
        vec_deque!($($value.into()),*)
    };
}
