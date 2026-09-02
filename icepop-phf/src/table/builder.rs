//! The mutable staging table, and the minimal perfect hash function construction.
//!
//! Entries live in a `Vec` in insertion order; a `hashbrown` side table maps a key's hash to its
//! index in that `Vec`, so the builder behaves like an ordinary hash table while entries are being
//! collected. Freezing happens in [`build_ordered`], which is the only expensive step.
//!
//! Indices are `u32`, which caps a table at `u32::MAX - 1` entries. Every insertion path asserts
//! the cap rather than silently truncating.

use crate::portability::{
    BuildHasherOps, EqOps, HashOps, HasherOps, MapOps, NonPortable, OrderedMapOps, OrderedSetOps,
    SetOps, TableOps,
};
use crate::table::Table;

use hashbrown::hash_table::{Entry, HashTable};

use portable::DefaultHasherSeed;

use core::marker::PhantomData;

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Entries in insertion order, plus a hash index into them.
///
/// `table` holds `(entry index, hash)` pairs; the cached hash lets the table be resized and
/// rehashed without touching the keys.
pub struct Builder<O: TableOps, S = DefaultHasherSeed, P = NonPortable> {
    entries: Vec<O::Entry>,
    table: HashTable<(u32, u64)>,
    hasher_builder: S,
    _portable: PhantomData<P>,
}

/// The pieces of a built unordered table, before they are wrapped in a
/// [`Table`](super::Table).
pub struct TableImpl<T, S> {
    pub hasher_builder: S,
    pub entries: Box<[T]>,
    pub global_param: u8,
    pub params: Box<[u32]>,
}

/// The pieces of a built ordered table, before they are wrapped in a
/// [`Table`](super::Table).
///
/// Differs from [`TableImpl`] only by `indices`, the slot-to-entry map that keeping insertion
/// order requires.
pub struct OrderedTableImpl<T, S> {
    pub hasher_builder: S,
    pub entries: Box<[T]>,
    pub global_param: u8,
    pub params: Box<[u32]>,
    pub indices: Box<[u32]>,
}

impl<O: TableOps, S, P> Builder<O, S, P> {
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            entries: Vec::new(),
            table: HashTable::new(),
            hasher_builder: hasher,
            _portable: PhantomData,
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        let capacity = capacity.min(u32::MAX as usize);
        Self {
            entries: Vec::with_capacity(capacity),
            table: HashTable::with_capacity(capacity),
            hasher_builder: hasher,
            _portable: PhantomData,
        }
    }

    pub fn hasher(&self) -> &S {
        &self.hasher_builder
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity().min(self.table.capacity())
    }

    pub fn iter(&self) -> core::slice::Iter<'_, O::Entry> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, O::Entry> {
        self.entries.iter_mut()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.table.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
        self.table.reserve(additional, move |&(_, h)| h);
    }

    pub fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.table.shrink_to_fit(move |&(_, h)| h);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.entries.shrink_to(min_capacity);
        self.table.shrink_to(min_capacity, move |&(_, h)| h);
    }

    pub fn as_slice(&self) -> &[O::Entry] {
        &self.entries
    }
}

impl<O, S, P> Builder<O, S, P>
where
    O: TableOps,
    S: BuildHasherOps<P>,
{
    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let hash = self.hasher_builder.hash_one(key);
        self.table
            .find(hash, |&(idx, _)| {
                key.eq(O::get_key(&self.entries[idx as usize]))
            })
            .is_some()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let hash = self.hasher_builder.hash_one(key);
        self.table
            .find(hash, |&(idx, _)| {
                key.eq(O::get_key(&self.entries[idx as usize]))
            })
            .map(|&(idx, _)| &self.entries[idx as usize])
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let hash = self.hasher_builder.hash_one(key);
        self.table
            .find(hash, |&(idx, _)| {
                key.eq(O::get_key(&self.entries[idx as usize]))
            })
            .map(|&(idx, _)| &mut self.entries[idx as usize])
    }

    /// # Panics
    ///
    /// Panics if `default()` produces an entry whose key is not equal to `key`, since the table
    /// would then be unable to find it again, or if the table is already at its entry cap.
    pub fn get_or_insert_with<Q, F>(&mut self, key: &Q, default: F) -> &O::Entry
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
        F: FnOnce() -> O::Entry,
    {
        let hash = self.hasher_builder.hash_one(key);
        let entries = &mut self.entries;
        let table = &mut self.table;

        match table.entry(
            hash,
            |&(idx, _)| key.eq(O::get_key(&entries[idx as usize])),
            |&(_, h)| h,
        ) {
            Entry::Vacant(entry) => {
                assert_ne!(
                    entries.len(),
                    u32::MAX as usize,
                    "table exceeds maximum entries",
                );

                let value = default();
                assert!(
                    key.eq(O::get_key(&value)),
                    "default value does not match key"
                );

                let idx = entries.len() as u32;
                entries.push(value);
                entry.insert((idx, hash));

                &entries[idx as usize]
            }
            Entry::Occupied(entry) => &entries[entry.get().0 as usize],
        }
    }
}

impl<O, S, P> Builder<O, S, P>
where
    O: TableOps,
    S: BuildHasherOps<P>,
    O::Key: HashOps<S, P> + EqOps<O::Key, P>,
{
    /// # Panics
    ///
    /// Panics if the table is already at its entry cap.
    pub fn replace(&mut self, mut value: O::Entry) -> Option<O::Entry> {
        let hash = self.hasher_builder.hash_one(O::get_key(&value));
        let entries = &mut self.entries;
        let table = &mut self.table;

        match table.entry(
            hash,
            |&(idx, _)| O::get_key(&entries[idx as usize]).eq(O::get_key(&value)),
            |&(_, h)| h,
        ) {
            Entry::Vacant(entry) => {
                assert_ne!(
                    entries.len(),
                    u32::MAX as usize,
                    "table exceeds maximum number of entries",
                );

                let idx = entries.len() as u32;
                entries.push(value);
                entry.insert((idx, hash));
                None
            }
            Entry::Occupied(entry) => {
                let idx = entry.get().0 as usize;

                // SAFETY: `idx` was read from the index table, which only ever holds indices of
                // entries currently in `entries`, so it is in bounds; stating that lets the access
                // below skip a bounds check.
                unsafe {
                    core::hint::assert_unchecked(idx < entries.len());
                }
                core::mem::swap(&mut entries[idx], &mut value);
                Some(value)
            }
        }
    }

    /// # Panics
    ///
    /// Panics if the table is already at its entry cap.
    pub fn insert(&mut self, value: O::Entry) -> bool {
        let hash = self.hasher_builder.hash_one(O::get_key(&value));
        let entries = &mut self.entries;
        let table = &mut self.table;

        match table.entry(
            hash,
            |&(idx, _)| O::get_key(&entries[idx as usize]).eq(O::get_key(&value)),
            |&(_, h)| h,
        ) {
            Entry::Vacant(entry) => {
                assert_ne!(
                    entries.len(),
                    u32::MAX as usize,
                    "table exceeds maximum number of entries",
                );

                let idx = entries.len() as u32;
                entries.push(value);
                entry.insert((idx, hash));
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    /// # Panics
    ///
    /// Panics if the table is already at its entry cap.
    pub fn get_or_insert(&mut self, value: O::Entry) -> &O::Entry {
        let hash = self.hasher_builder.hash_one(O::get_key(&value));
        let entries = &mut self.entries;
        let table = &mut self.table;

        match table.entry(
            hash,
            |&(idx, _)| O::get_key(&value).eq(O::get_key(&entries[idx as usize])),
            |&(_, h)| h,
        ) {
            Entry::Vacant(entry) => {
                assert_ne!(
                    entries.len(),
                    u32::MAX as usize,
                    "table exceeds maximum entries",
                );

                let idx = entries.len() as u32;
                entries.push(value);
                entry.insert((idx, hash));

                &entries[idx as usize]
            }
            Entry::Occupied(entry) => &entries[entry.get().0 as usize],
        }
    }

    /// Removes an entry, closing the hole it leaves in two different ways.
    ///
    /// An ordered table shifts the tail down and decrements every larger index, keeping insertion
    /// order at `O(n)`. An unordered one swaps the last entry into the hole and repoints just that
    /// entry's index, which is `O(1)` but reorders.
    pub fn take<Q>(&mut self, key: &Q) -> Option<O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let hash = self.hasher_builder.hash_one(key);
        let hasher_builder = &self.hasher_builder;
        let entries = &mut self.entries;
        let table = &mut self.table;

        if let Ok(entry) =
            table.find_entry(hash, |&(idx, _)| key.eq(O::get_key(&entries[idx as usize])))
        {
            if O::HAVE_INDICES {
                let idx = entry.remove().0.0;
                for (i, _) in table {
                    if *i > idx {
                        *i -= 1;
                    }
                }
                Some(entries.remove(idx as usize))
            } else {
                let idx = entry.remove().0.0;
                let value = entries.swap_remove(idx as usize);
                let old_idx = entries.len() as u32;

                if idx < old_idx {
                    let hash = hasher_builder.hash_one(O::get_key(&entries[idx as usize]));
                    table.find_mut(hash, |&(i, _)| i == old_idx).unwrap().0 = idx;
                }

                Some(value)
            }
        } else {
            None
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        self.take(key).is_some()
    }

    fn build_ordered(self) -> OrderedTableImpl<O::Entry, S>
    where
        u8: HashOps<S, P>,
        u32: HashOps<S, P>,
    {
        build_ordered::<O, S, P>(self.entries, self.hasher_builder)
    }

    fn build_unordered(self) -> TableImpl<O::Entry, S>
    where
        u8: HashOps<S, P>,
        u32: HashOps<S, P>,
    {
        self.build_ordered().into_unordered()
    }
}

impl<K, V, O, S, P> Builder<O, S, P>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
    S: BuildHasherOps<P>,
{
    pub fn map_get<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        K: 'a,
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.get(key).map(|(_, value)| value)
    }

    pub fn map_get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V>
    where
        K: 'a,
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.get_mut(key).map(|(_, value)| value)
    }

    pub fn map_get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.get(key).map(|(k, v)| (k, v))
    }

    pub fn map_get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.get_mut(key).map(|(k, v)| (&*k, v))
    }
}

impl<K, V, O, S, P> Builder<O, S, P>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
    K: HashOps<S, P> + EqOps<K, P>,
    S: BuildHasherOps<P>,
{
    /// Replaces a value with one computed from the old value, moved out by value.
    ///
    /// The old value has to leave the `Vec` before `update` can consume it, which leaves the
    /// builder briefly inconsistent. `Guard` restores it if `update` panics: the entry stays
    /// removed, but the index table and the insertion order agree again.
    ///
    /// # Panics
    ///
    /// Panics if the table is already at its entry cap.
    pub fn map_upsert<F>(&mut self, key: K, update: F) -> bool
    where
        F: FnOnce(Option<V>) -> V,
    {
        let hash = self.hasher_builder.hash_one(&key);
        let hasher_builder = &self.hasher_builder;
        let entries = &mut self.entries;
        let table = &mut self.table;

        match table.entry(
            hash,
            |&(idx, _)| entries[idx as usize].0.eq(&key),
            |&(_, h)| h,
        ) {
            Entry::Vacant(entry) => {
                assert_ne!(
                    entries.len(),
                    u32::MAX as usize,
                    "table exceeds maximum entries",
                );

                let value = update(None);
                let idx = entries.len() as u32;
                entries.push((key, value));
                entry.insert((idx, hash));
                true
            }
            Entry::Occupied(entry) => {
                struct Guard<'a, K: HashOps<S, P>, V, S: BuildHasherOps<P>, O: TableOps, P> {
                    table: &'a mut HashTable<(u32, u64)>,
                    entries: &'a mut [(K, V)],
                    hasher_builder: &'a S,
                    idx: u32,
                    _ops: PhantomData<(O, P)>,
                }

                impl<K: HashOps<S, P>, V, S: BuildHasherOps<P>, O: TableOps, P> Drop for Guard<'_, K, V, S, O, P> {
                    fn drop(&mut self) {
                        let idx = self.idx;

                        if (idx as usize) >= self.entries.len() {
                            return;
                        }

                        if O::HAVE_INDICES {
                            self.entries[idx as usize..].rotate_left(1);
                            self.table.iter_mut().for_each(|(i, _)| {
                                if *i > idx {
                                    *i -= 1;
                                }
                            });
                        } else {
                            let old_idx = self.entries.len() as u32;
                            let hash = self.hasher_builder.hash_one(&self.entries[idx as usize].0);
                            self.table.find_mut(hash, |&(i, _)| i == old_idx).unwrap().0 = idx;
                        }
                    }
                }

                let (idx, h) = entry.remove().0;
                let idx = idx as usize;

                // SAFETY: `idx` was read from the index table, which only ever holds indices of
                // entries currently in `entries`, so it is in bounds; stating that lets the access
                // below skip a bounds check.
                unsafe { core::hint::assert_unchecked(idx < entries.len()) }
                let v = entries.swap_remove(idx).1;

                let guard = Guard::<'_, K, V, S, O, P> {
                    table: &mut *table,
                    entries: &mut *entries,
                    hasher_builder,
                    idx: idx as u32,
                    _ops: PhantomData,
                };
                let new_v = update(Some(v));
                core::mem::forget(guard);

                let mut entry = (key, new_v);
                if idx < entries.len() {
                    core::mem::swap(&mut entries[idx], &mut entry);
                }
                entries.push(entry);
                table.insert_unique(hash, (idx as u32, h), |&(_, h)| h);
                false
            }
        }
    }

    pub fn map_insert(&mut self, key: K, value: V) -> Option<V> {
        self.replace((key, value)).map(|(_, value)| value)
    }

    pub fn map_get_or_insert_with<'a, Q, F>(&'a mut self, key: &Q, default: F) -> &'a mut V
    where
        K: 'a,
        Q: HashOps<S, P> + EqOps<K, P> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
        F: FnOnce() -> V,
    {
        let hash = self.hasher_builder.hash_one(key);
        let entries = &mut self.entries;
        let table = &mut self.table;

        match table.entry(
            hash,
            |&(idx, _)| key.eq(&entries[idx as usize].0),
            |&(_, h)| h,
        ) {
            Entry::Vacant(entry) => {
                assert_ne!(
                    entries.len(),
                    u32::MAX as usize,
                    "table exceeds maximum entries",
                );

                let idx = entries.len() as u32;
                entries.push((key.to_owned(), default()));
                entry.insert((idx, hash));

                &mut entries[idx as usize].1
            }
            Entry::Occupied(entry) => &mut entries[entry.get().0 as usize].1,
        }
    }

    pub fn map_get_or_insert_default<'a, Q>(&'a mut self, key: &Q) -> &'a mut V
    where
        K: 'a,
        Q: HashOps<S, P> + EqOps<K, P> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
        V: Default,
    {
        self.map_get_or_insert_with(key, V::default)
    }

    pub fn map_get_or_insert<'a, Q>(&'a mut self, key: &Q, default: V) -> &'a mut V
    where
        K: 'a,
        Q: HashOps<S, P> + EqOps<K, P> + alloc::borrow::ToOwned<Owned = K> + ?Sized,
    {
        self.map_get_or_insert_with(key, move || default)
    }

    pub fn map_remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        self.take(key).map(|(_, v)| v)
    }
}

impl<T, S, P> Builder<OrderedSetOps<T>, S, P>
where
    S: BuildHasherOps<P>,
    T: HashOps<S, P> + EqOps<T, P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    pub fn build(self) -> Table<OrderedSetOps<T>, S, P> {
        let OrderedTableImpl {
            hasher_builder,
            entries,
            global_param,
            params,
            indices,
        } = self.build_ordered();

        Table {
            global_param,
            params,
            indices,
            entries,
            hasher_builder,
            _portable: PhantomData,
        }
    }
}

impl<T, S, P> Builder<SetOps<T>, S, P>
where
    S: BuildHasherOps<P>,
    T: HashOps<S, P> + EqOps<T, P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    pub fn build(self) -> Table<SetOps<T>, S, P> {
        let TableImpl {
            hasher_builder,
            entries,
            global_param,
            params,
        } = self.build_unordered();

        Table {
            global_param,
            params,
            indices: (),
            entries,
            hasher_builder,
            _portable: PhantomData,
        }
    }
}

impl<K, V, S, P> Builder<OrderedMapOps<K, V>, S, P>
where
    S: BuildHasherOps<P>,
    K: HashOps<S, P> + EqOps<K, P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    pub fn build(self) -> Table<OrderedMapOps<K, V>, S, P> {
        let OrderedTableImpl {
            hasher_builder,
            entries,
            global_param,
            params,
            indices,
        } = self.build_ordered();

        Table {
            global_param,
            params,
            indices,
            entries,
            hasher_builder,
            _portable: PhantomData,
        }
    }
}

impl<K, V, S, P> Builder<MapOps<K, V>, S, P>
where
    S: BuildHasherOps<P>,
    K: HashOps<S, P> + EqOps<K, P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    pub fn build(self) -> Table<MapOps<K, V>, S, P> {
        let TableImpl {
            hasher_builder,
            entries,
            global_param,
            params,
        } = self.build_unordered();

        Table {
            global_param,
            params,
            indices: (),
            entries,
            hasher_builder,
            _portable: PhantomData,
        }
    }
}

/// Constructs the minimal perfect hash function over `entries`.
///
/// The CHD construction, in three phases:
///
/// 1. **Seed.** Hash every key under `global_param`. If two keys collide in 64 bits no
///    displacement can separate them, so bump `global_param` and start over.
/// 2. **Bucket.** Reduce each key's hash modulo `len` and group keys by the result. Sort the
///    buckets largest first, so the hardest ones are placed while slots are still plentiful.
/// 3. **Displace.** For each bucket in turn, search for a displacement `d` that sends all of
///    its keys to free slots when fed to the same hasher after the key. A conflict releases
///    the slots claimed for that bucket and retries with `d + 1`. Exhausting `d` means this
///    seed cannot be completed, so `global_param` is bumped and everything restarts.
///
/// Since there are exactly as many slots as entries and every entry claims one, the result is
/// both perfect and minimal, and `indices` ends up a permutation of `0..len`.
///
/// # Panics
///
/// Panics once `global_param` is exhausted, in either phase. Reaching that with a sound hasher
/// is not realistic; it means the hasher collides on distinct keys or clusters them badly.
fn build_ordered<O, S, P>(
    entries: Vec<O::Entry>,
    hasher_builder: S,
) -> OrderedTableImpl<O::Entry, S>
where
    O: TableOps,
    S: BuildHasherOps<P>,
    O::Key: HashOps<S, P> + EqOps<O::Key, P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    let len = entries.len();

    let mut global_param = 0u8;
    let mut hashes = hashbrown::HashSet::with_capacity(entries.len());
    let mut params = alloc::vec![0u32; len];
    let mut indices = alloc::vec![len as u32; len];
    let mut buckets: Vec<Vec<u32>> = core::iter::repeat_with(Vec::new).take(len).collect();
    let mut sorted_buckets: Vec<_> = (0..len).collect();
    let mut slots: Vec<u32> = Vec::with_capacity(len);

    'reseed: loop {
        'find_global: loop {
            for entry in &entries {
                let mut hasher = hasher_builder.build_hasher();
                HashOps::<S, P>::hash(&global_param, &mut hasher);
                O::get_key(entry).hash(&mut hasher);
                if !hashes.insert(hasher.finish()) {
                    if global_param == u8::MAX {
                        panic!(
                            "minimal perfect hash function cannot be constructed due to unavoidable 64-bit hash collisions - likely bad `Hasher` or `Hash` implementation"
                        );
                    }
                    hashes.clear();
                    global_param += 1;
                    continue 'find_global;
                }
            }
            break;
        }

        for (idx, entry) in entries.iter().enumerate() {
            let mut hasher = hasher_builder.build_hasher();
            HashOps::<S, P>::hash(&global_param, &mut hasher);
            O::get_key(entry).hash(&mut hasher);
            let hash = hasher.finish();
            buckets[(hash % (len as u64)) as usize].push(idx as u32);
        }

        sorted_buckets.sort_by(|&l, &r| Ord::cmp(&buckets[r].len(), &buckets[l].len()));

        for idx in 0..len {
            let bucket = &buckets[sorted_buckets[idx]];
            if bucket.is_empty() {
                continue;
            }

            let mut d = 0u32;
            let mut item = 0usize;
            slots.clear();

            while item < bucket.len() {
                let mut hasher = hasher_builder.build_hasher();
                HashOps::<S, P>::hash(&global_param, &mut hasher);
                O::get_key(&entries[bucket[item] as usize]).hash(&mut hasher);
                HashOps::<S, P>::hash(&d, &mut hasher);
                let hash = hasher.finish();

                let slot = (hash % (len as u64)) as u32;
                if indices[slot as usize] != (len as u32) {
                    if d == u32::MAX {
                        if global_param == u8::MAX {
                            panic!(
                                "minimal perfect hash function cannot be constructed because no displacement placed a bucket's keys in free slots, for any global parameter - likely a poorly distributed `Hasher`"
                            );
                        }
                        global_param += 1;
                        hashes.clear();
                        params.fill(0u32);
                        indices.fill(len as u32);
                        buckets.iter_mut().for_each(|bucket| bucket.clear());
                        sorted_buckets
                            .iter_mut()
                            .enumerate()
                            .for_each(|(idx, val)| {
                                *val = idx;
                            });
                        slots.clear();

                        continue 'reseed;
                    }
                    d += 1;
                    item = 0;
                    for s in &slots {
                        indices[*s as usize] = len as u32;
                    }
                    slots.clear();
                } else {
                    slots.push(slot);
                    indices[slot as usize] = bucket[item];
                    item += 1;
                }

                params[sorted_buckets[idx]] = d;
            }
        }

        break;
    }

    OrderedTableImpl {
        hasher_builder,
        entries: entries.into(),
        global_param,
        params: params.into(),
        indices: indices.into(),
    }
}

impl<T, S> OrderedTableImpl<T, S> {
    /// Applies `indices` to `entries` in place, so that a slot is its own entry index.
    ///
    /// `indices` is a permutation, so it can be applied by walking each cycle and swapping along
    /// it, which is what lets the unordered tables drop the array entirely and save four bytes per
    /// entry plus an indirection per lookup. Insertion order is lost.
    fn into_unordered(self) -> TableImpl<T, S> {
        let Self {
            hasher_builder,
            mut entries,
            global_param,
            params,
            mut indices,
        } = self;

        let len = entries.len();
        for start in 0..len {
            if indices[start] as usize == start {
                continue;
            }

            let mut slot = start;
            loop {
                let src = indices[slot] as usize;
                indices[slot] = slot as u32;
                if src == start {
                    break;
                }
                entries.swap(slot, src);
                slot = src;
            }
        }

        TableImpl {
            hasher_builder,
            entries,
            global_param,
            params,
        }
    }
}

impl<O, S, P> Clone for Builder<O, S, P>
where
    O: TableOps,
    O::Entry: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            table: self.table.clone(),
            hasher_builder: self.hasher_builder.clone(),
            _portable: PhantomData,
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.entries.clone_from(&other.entries);
        self.table.clone_from(&other.table);
        self.hasher_builder.clone_from(&other.hasher_builder);
    }
}

impl<O, S, P> IntoIterator for Builder<O, S, P>
where
    O: TableOps,
{
    type Item = O::Entry;
    type IntoIter = alloc::vec::IntoIter<O::Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}
