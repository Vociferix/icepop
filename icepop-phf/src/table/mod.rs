use crate::portability::{
    BuildHasherOps, EqOps, HashOps, HasherOps, NonPortable, Portable, TableOps,
};

use portable::DefaultHasherSeed;

use core::marker::PhantomData;

use alloc::boxed::Box;

#[cfg(feature = "rkyv")]
pub mod rkyv;

mod builder;

pub use builder::{Builder, OrderedTableImpl, TableImpl};

pub struct Table<O: TableOps, S = DefaultHasherSeed, P = NonPortable> {
    pub(crate) global_param: u8,
    pub(crate) params: Box<[u32]>,
    pub(crate) indices: O::Indices,
    pub(crate) entries: Box<[O::Entry]>,
    pub(crate) hasher_builder: S,
    pub(crate) _portable: PhantomData<P>,
}

impl<O, S, P> Table<O, S, P>
where
    O: TableOps,
{
    pub fn hasher(&self) -> &S {
        &self.hasher_builder
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> core::slice::Iter<'_, O::Entry> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, O::Entry> {
        self.entries.iter_mut()
    }

    pub fn as_slice(&self) -> &[O::Entry] {
        &self.entries
    }

    pub fn index(&self, index: usize) -> Option<&O::Entry> {
        self.entries.get(index)
    }

    pub fn index_mut(&mut self, index: usize) -> Option<&mut O::Entry> {
        self.entries.get_mut(index)
    }

    pub unsafe fn index_unchecked(&self, index: usize) -> &O::Entry {
        unsafe { self.entries.get_unchecked(index) }
    }

    pub unsafe fn index_unchecked_mut(&mut self, index: usize) -> &mut O::Entry {
        unsafe { self.entries.get_unchecked_mut(index) }
    }
}

impl<K, V, O, S, P> Table<O, S, P>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
{
    pub fn map_index(&self, index: usize) -> Option<(&K, &V)> {
        self.index(index).map(|(k, v)| (k, v))
    }

    pub fn map_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        self.index_mut(index).map(|(k, v)| (&*k, v))
    }

    pub unsafe fn map_index_unchecked(&self, index: usize) -> (&K, &V) {
        let (k, v) = unsafe { self.index_unchecked(index) };
        (k, v)
    }

    pub unsafe fn map_index_unchecked_mut(&mut self, index: usize) -> (&K, &mut V) {
        let (k, v) = unsafe { self.index_unchecked_mut(index) };
        (&*k, v)
    }
}

impl<O, S, P> Table<O, S, P>
where
    O: TableOps,
    S: BuildHasherOps<P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
    pub unsafe fn get_index_unchecked<Q>(&self, key: &Q) -> usize
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let modulus = self.entries.len() as u64;
        unsafe {
            core::hint::assert_unchecked(modulus != 0);
        }

        let mut hasher = self.hasher_builder.build_hasher();

        HashOps::<S, P>::hash(&self.global_param, &mut hasher);
        key.hash(&mut hasher);
        let param_idx = (hasher.finish() % modulus) as usize;
        let param = unsafe { *self.params.get_unchecked(param_idx) };

        HashOps::<S, P>::hash(&param, &mut hasher);
        let index_idx = (hasher.finish() % modulus) as usize;
        unsafe { O::get_index(&self.indices, index_idx) }
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.eq(O::get_key(entry)).then_some(index)
    }

    pub fn contains<Q>(&self, key: &Q) -> bool
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return false;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.eq(O::get_key(entry))
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked(index) };
        key.eq(O::get_key(entry)).then_some(entry)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut O::Entry>
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        if self.entries.is_empty() {
            return None;
        }

        let index = unsafe { self.get_index_unchecked(key) };
        let entry = unsafe { self.entries.get_unchecked_mut(index) };
        key.eq(O::get_key(entry)).then_some(entry)
    }

    pub unsafe fn get_unchecked<Q>(&self, key: &Q) -> &O::Entry
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let idx = unsafe { self.get_index_unchecked(key) };
        unsafe { self.entries.get_unchecked(idx) }
    }

    pub unsafe fn get_unchecked_mut<Q>(&mut self, key: &Q) -> &mut O::Entry
    where
        Q: HashOps<S, P> + EqOps<O::Key, P> + ?Sized,
    {
        let idx = unsafe { self.get_index_unchecked(key) };
        unsafe { self.entries.get_unchecked_mut(idx) }
    }
}

impl<K, V, O, S, P> Table<O, S, P>
where
    O: TableOps<Entry = (K, V), Key = K, Value = V>,
    S: BuildHasherOps<P>,
    u8: HashOps<S, P>,
    u32: HashOps<S, P>,
{
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

    pub fn map_get<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        self.get(key).map(|(_, v)| v)
    }

    pub fn map_get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V>
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        self.get_mut(key).map(|(_, v)| v)
    }

    pub unsafe fn map_get_key_value_unchecked<Q>(&self, key: &Q) -> (&K, &V)
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        let (k, v) = unsafe { self.get_unchecked(key) };
        (k, v)
    }

    pub unsafe fn map_get_key_value_unchecked_mut<Q>(&mut self, key: &Q) -> (&K, &mut V)
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
    {
        let (k, v) = unsafe { self.get_unchecked_mut(key) };
        (&*k, v)
    }

    pub unsafe fn map_get_unchecked<'a, Q>(&'a self, key: &Q) -> &'a V
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        &unsafe { self.get_unchecked(key) }.1
    }

    pub unsafe fn map_get_unchecked_mut<'a, Q>(&'a mut self, key: &Q) -> &'a mut V
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        &mut unsafe { self.get_unchecked_mut(key) }.1
    }

    pub fn map_get_disjoint_key_value_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<(&'a K, &'a mut V)>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        let indices = keys.map(|key| self.get_index(key));
        assert!(unique_indices(&indices), "duplicate keys found");
        indices.map(|idx| {
            idx.map(|idx| {
                let (k, v) = unsafe { &mut *self.entries.as_mut_ptr().add(idx) };
                (&*k, v)
            })
        })
    }

    pub fn map_get_disjoint_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<&'a mut V>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        let indices = keys.map(|key| self.get_index(key));
        assert!(unique_indices(&indices), "duplicate key found");
        indices.map(|idx| idx.map(|idx| unsafe { &mut (*self.entries.as_mut_ptr().add(idx)).1 }))
    }

    pub unsafe fn map_get_disjoint_key_value_unchecked_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<(&'a K, &'a mut V)>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        keys.map(|key| {
            self.get_index(key).map(|idx| {
                let (k, v) = unsafe { &mut *self.entries.as_mut_ptr().add(idx) };
                (&*k, v)
            })
        })
    }

    pub unsafe fn map_get_disjoint_unchecked_mut<'a, Q, const N: usize>(
        &'a mut self,
        keys: [&Q; N],
    ) -> [Option<&'a mut V>; N]
    where
        Q: HashOps<S, P> + EqOps<K, P> + ?Sized,
        K: 'a,
    {
        keys.map(|key| {
            self.get_index(key)
                .map(|idx| unsafe { &mut (*self.entries.as_mut_ptr().add(idx)).1 })
        })
    }
}

impl<O, S, P> Default for Table<O, S, P>
where
    O: TableOps,
    O::Indices: Default,
    S: Default,
{
    fn default() -> Self {
        Self {
            global_param: 0,
            params: Box::default(),
            indices: <O::Indices as Default>::default(),
            entries: Box::default(),
            hasher_builder: S::default(),
            _portable: PhantomData,
        }
    }
}

impl<O, S, P> Clone for Table<O, S, P>
where
    O: TableOps,
    O::Indices: Clone,
    O::Entry: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            global_param: self.global_param,
            params: self.params.clone(),
            indices: self.indices.clone(),
            entries: self.entries.clone(),
            hasher_builder: self.hasher_builder.clone(),
            _portable: PhantomData,
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.global_param = other.global_param;
        self.params.clone_from(&other.params);
        self.indices.clone_from(&other.indices);
        self.entries.clone_from(&other.entries);
        self.hasher_builder.clone_from(&other.hasher_builder);
    }
}

impl<O: TableOps, S, P> IntoIterator for Table<O, S, P> {
    type Item = O::Entry;
    type IntoIter = alloc::vec::IntoIter<O::Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<O: TableOps, S1, S2, P1, P2> PartialEq<Table<O, S2, P2>> for Table<O, S1, P1>
where
    O::Entry: PartialEq,
    O::Key: HashOps<S2, P2> + EqOps<O::Key, P2>,
    S1: BuildHasherOps<P1>,
    S2: BuildHasherOps<P2>,
    u32: HashOps<S2, P2>,
    u8: HashOps<S2, P2>,
{
    fn eq(&self, other: &Table<O, S2, P2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for entry in self.iter() {
            let Some(other) = other.get(O::get_key(entry)) else {
                return false;
            };
            if entry != other {
                return false;
            }
        }

        true
    }
}

impl<O: TableOps, S, P> Eq for Table<O, S, P>
where
    O::Entry: Eq,
    O::Key: HashOps<S, P> + EqOps<O::Key, P>,
    S: BuildHasherOps<P>,
    u32: HashOps<S, P>,
    u8: HashOps<S, P>,
{
}

#[inline]
fn unique_indices(mut indices: &[Option<usize>]) -> bool {
    while let Some((&first, rest)) = indices.split_first() {
        if let Some(first) = first
            && rest.iter().filter_map(|idx| *idx).any(|idx| first == idx)
        {
            return false;
        }
        indices = rest;
    }
    true
}

#[cfg(feature = "serde")]
impl<O, H> serde::Serialize for Table<O, H, Portable>
where
    O: TableOps,
    O::Entry: serde::Serialize,
    O::Indices: serde::Serialize,
    H: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut ser =
            serializer.serialize_struct("Table", 5 - (if O::HAVE_INDICES { 0 } else { 1 }))?;
        ser.serialize_field("hasher", &self.hasher_builder)?;
        ser.serialize_field("entries", &self.entries)?;
        ser.serialize_field("global_param", &self.global_param)?;
        ser.serialize_field("params", &self.params)?;
        if O::HAVE_INDICES {
            ser.serialize_field("indices", &self.indices)?;
        }
        ser.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, O, S> serde::Deserialize<'de> for Table<O, S, Portable>
where
    O: TableOps,
    O::Entry: serde::Deserialize<'de>,
    O::Indices: serde::Deserialize<'de> + Default,
    S: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<O, S>(core::marker::PhantomData<fn() -> (PhantomData<O>, S)>);

        enum Field {
            Hasher,
            Entries,
            GlobalParam,
            Params,
            Indices,
        }

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl serde::de::Visitor<'_> for Visitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        f.write_str("one of \"global_param\", \"params\", \"indices\", \"entries\", or \"hasher\"")
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            0 => Ok(Field::Hasher),
                            1 => Ok(Field::Entries),
                            2 => Ok(Field::GlobalParam),
                            3 => Ok(Field::Params),
                            4 => Ok(Field::Indices),
                            _ => Err(E::invalid_value(serde::de::Unexpected::Unsigned(v), &self)),
                        }
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            0 => Ok(Field::Hasher),
                            1 => Ok(Field::Entries),
                            2 => Ok(Field::GlobalParam),
                            3 => Ok(Field::Params),
                            4 => Ok(Field::Indices),
                            _ => Err(E::invalid_value(serde::de::Unexpected::Signed(v), &self)),
                        }
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match v {
                            "hasher" => Ok(Field::Hasher),
                            "entries" => Ok(Field::Entries),
                            "global_param" => Ok(Field::GlobalParam),
                            "params" => Ok(Field::Params),
                            "indices" => Ok(Field::Indices),
                            _ => Err(E::invalid_value(serde::de::Unexpected::Str(v), &self)),
                        }
                    }
                }

                deserializer.deserialize_identifier(Visitor)
            }
        }

        impl<'de, O, S> serde::de::Visitor<'de> for Visitor<O, S>
        where
            O: TableOps,
            O::Entry: serde::Deserialize<'de>,
            O::Indices: serde::Deserialize<'de> + Default,
            S: serde::Deserialize<'de>,
        {
            type Value = Table<O, S, Portable>;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a phf table layout")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let hasher_builder: S = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("hasher"))?;
                let entries: Box<[O::Entry]> = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("entries"))?;
                let global_param: u8 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("global_param"))?;
                let params: Box<[u32]> = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::missing_field("params"))?;
                let indices: O::Indices = if O::HAVE_INDICES {
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::missing_field("indices"))?
                } else {
                    <O::Indices as Default>::default()
                };

                let table = Table {
                    global_param,
                    params,
                    indices,
                    entries,
                    hasher_builder,
                    _portable: PhantomData,
                };

                if !O::verify(&table) {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other("phf table has invalid layout"),
                        &self,
                    ));
                }

                Ok(table)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut hasher_builder: Option<S> = None;
                let mut entries: Option<Box<[O::Entry]>> = None;
                let mut global_param: Option<u8> = None;
                let mut params: Option<Box<[u32]>> = None;
                let mut indices: Option<O::Indices> = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::GlobalParam => {
                            if global_param.is_some() {
                                return Err(serde::de::Error::duplicate_field("global_param"));
                            }
                            global_param = Some(map.next_value()?);
                        }
                        Field::Params => {
                            if params.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params = Some(map.next_value()?);
                        }
                        Field::Indices if O::HAVE_INDICES => {
                            if indices.is_some() {
                                return Err(serde::de::Error::duplicate_field("indices"));
                            }
                            indices = Some(map.next_value()?);
                        }
                        Field::Indices => {
                            return Err(serde::de::Error::unknown_field(
                                "indices",
                                &["params", "entries", "hasher"],
                            ));
                        }
                        Field::Entries => {
                            if entries.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries = Some(map.next_value()?);
                        }
                        Field::Hasher => {
                            if hasher_builder.is_some() {
                                return Err(serde::de::Error::duplicate_field("hasher"));
                            }
                            hasher_builder = Some(map.next_value()?);
                        }
                    }
                }

                let Some(hasher_builder) = hasher_builder else {
                    return Err(serde::de::Error::missing_field("hasher"));
                };
                let Some(entries) = entries else {
                    return Err(serde::de::Error::missing_field("entries"));
                };
                let Some(global_param) = global_param else {
                    return Err(serde::de::Error::missing_field("global_param"));
                };
                let Some(params) = params else {
                    return Err(serde::de::Error::missing_field("params"));
                };
                let indices = if O::HAVE_INDICES {
                    let Some(indices) = indices else {
                        return Err(serde::de::Error::missing_field("indices"));
                    };
                    indices
                } else {
                    <O::Indices as Default>::default()
                };

                let table = Table {
                    global_param,
                    params,
                    indices,
                    entries,
                    hasher_builder,
                    _portable: PhantomData,
                };

                if !O::verify(&table) {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Other("phf table has invalid layout"),
                        &self,
                    ));
                }

                Ok(table)
            }
        }

        deserializer.deserialize_struct(
            "Table",
            if O::HAVE_INDICES {
                &["hasher", "entries", "global_param", "params", "indices"]
            } else {
                &["hasher", "entries", "global_param", "params"]
            },
            Visitor(core::marker::PhantomData),
        )
    }
}
