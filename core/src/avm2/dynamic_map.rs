use crate::avm2::object::Object;
use crate::string::AvmString;
use fnv::FnvBuildHasher;
use gc_arena::Collect;
use hashbrown::HashTable;
use hashbrown::hash_table::Entry;
use std::cell::Cell;
use std::hash::{BuildHasher, Hash};

#[derive(Debug, Collect, Copy, Clone)]
#[collect(no_drop)]
pub struct DynamicProperty<V> {
    pub value: V,
    pub enumerable: bool,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum DynamicKey<'gc> {
    String(AvmString<'gc>),
    // When the name parses as a non-negative integer, we use that integer as the key
    // See `ScriptObject::get_property_local` and `ScriptObject::set_property_local`.
    // This is observable when iterating over the object keys, as the key
    // can be `number`
    Uint(u32),
    Object(Object<'gc>),
}

/// A HashMap designed for dynamic properties on an object.
///
/// Uses `HashTable` directly to expose stable bucket indices, which are
/// needed for correct iteration when entries are added or removed mid-iteration.
#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct DynamicMap<K, V> {
    table: HashTable<(K, DynamicProperty<V>)>,
    #[collect(require_static)]
    hasher: FnvBuildHasher,
    // The last index that was given back to flash
    #[collect(require_static)]
    public_index: Cell<usize>,
    // The actual bucket index that represents where an item is in the table
    #[collect(require_static)]
    real_index: Cell<usize>,
    // How many keys were added or removed so far; the indices above are as of `cached_at` of them.
    #[collect(require_static)]
    mutations: u64,
    #[collect(require_static)]
    cached_at: Cell<u64>,
}

impl<K: Eq + Hash, V> Default for DynamicMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash, V> DynamicMap<K, V> {
    fn hash_key(&self, key: &K) -> u64 {
        self.hasher.hash_one(key)
    }

    pub fn new() -> Self {
        Self {
            table: HashTable::new(),
            hasher: FnvBuildHasher::default(),
            public_index: Cell::new(0),
            real_index: Cell::new(0),
            mutations: 0,
            cached_at: Cell::new(0),
        }
    }

    pub fn get(&self, key: &K) -> Option<&DynamicProperty<V>> {
        let hash = self.hash_key(key);

        self.table.find(hash, |(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let hash = self.hash_key(key);

        self.table.find(hash, |(k, _)| k == key).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &DynamicProperty<V>)> {
        self.table.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.table.iter().map(|(k, _)| k)
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Gets the bucket index from the current public index.
    /// Returns `None` if the index is out of bounds.
    fn public_to_real_index(&self, index: usize) -> Option<usize> {
        let mut count = 0;
        let num_buckets = self.table.num_buckets();

        for i in 0..num_buckets {
            if let Some((_, v)) = self.table.get_bucket(i)
                && v.enumerable
            {
                count += 1;

                if count >= index {
                    return Some(i);
                }
            }
        }

        None
    }

    pub fn insert(&mut self, key: K, new_value: V) {
        let hash = self.hash_key(&key);

        match self
            .table
            .entry(hash, |(k, _)| *k == key, |(k, _)| self.hasher.hash_one(k))
        {
            Entry::Occupied(mut occupied) => {
                // NOTE: When inserting a new value into an already-occupied entry,
                // the value of the `enumerable` field isn't reset to `true`
                occupied.get_mut().1.value = new_value;
            }
            Entry::Vacant(vacant) => {
                vacant.insert((
                    key,
                    DynamicProperty {
                        value: new_value,
                        enumerable: true,
                    },
                ));
                self.mutations = self.mutations.wrapping_add(1);
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<DynamicProperty<V>> {
        let hash = self.hash_key(key);

        match self.table.find_entry(hash, |(k, _)| k == key) {
            Ok(occupied) => {
                let removed = occupied.remove().0.1;
                self.mutations = self.mutations.wrapping_add(1);
                Some(removed)
            }
            Err(_) => None,
        }
    }

    // NOTE: Per the docs on `HashTable::find_bucket_index`, bucket indices are
    // "only meaningful as long as the table is not resized and no entries are
    // added or removed". The current implementation uses tombstones and never
    // moves elements, so indices of other entries are preserved on removal in
    // practice, but this is not a guaranteed API contract and should not be
    // relied on long-term.
    // https://docs.rs/hashbrown/0.16.1/hashbrown/struct.HashTable.html#method.find_bucket_index
    pub fn next(&self, index: usize) -> Option<usize> {
        // Whatever this leaves in the cache is as of now. A key added or removed during a for-in
        // doesn't move where `next()` continues from (it carries on from the cached bucket, as
        // avmplus does), but it does move the key at each index, which `pair_at` must not trust.
        self.cached_at.set(self.mutations);
        // Start iteration from the beginning
        if index == 0 {
            if let Some(real) = self.public_to_real_index(1) {
                self.real_index.set(real);
                self.public_index.set(1);
                return Some(1);
            } else {
                self.public_index.set(0);
                self.real_index.set(0);
                return None;
            }
        }
        // If the public index is zero than this is the first time this is being enumerated,
        // if index != self.public_index, then we are enumerating twice OR we mutated while enumerating.
        //
        // Regardless of the reason, we just need to sync the supplied index to the real index.
        if self.public_index.get() == 0 || index != self.public_index.get() {
            if let Some(real) = self.public_to_real_index(index) {
                // Pick up where we left off in the iteration
                // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/avmplusHashtable.cpp#L395
                self.real_index.set(real);
                self.public_index.set(index);
            } else {
                self.public_index.set(0);
                self.real_index.set(0);
                return None;
            }
        }

        let real = self.real_index.get() + 1;
        let num_buckets = self.table.num_buckets();

        if !self.table.is_empty() && real < num_buckets {
            for i in real..num_buckets {
                if let Some((_, v)) = self.table.get_bucket(i)
                    && v.enumerable
                {
                    self.real_index.set(i);
                    self.public_index.set(self.public_index.get() + 1);
                    return Some(self.public_index.get());
                }
            }
        }

        None
    }

    pub fn pair_at(&self, index: usize) -> Option<&(K, DynamicProperty<V>)> {
        // The cache holds the key at `public_index` only if no key was added or removed since
        // `next()` set it. A reader by index without `next()`, as JSON.stringify is, would
        // otherwise get the stale key at that one index: one key listed twice, and one lost.
        let real_index = if self.public_index.get() == 0
            || self.public_index.get() != index
            || self.cached_at.get() != self.mutations
        {
            self.public_to_real_index(index)?
        } else {
            self.real_index.get()
        };

        self.table.get_bucket(real_index)
    }

    pub fn key_at(&self, index: usize) -> Option<&K> {
        self.pair_at(index).map(|(k, _)| k)
    }

    pub fn value_at(&self, index: usize) -> Option<&V> {
        self.pair_at(index).map(|(_, p)| &p.value)
    }

    pub fn set_enumerable(&mut self, key: &K, enumerable: bool) {
        let hash = self.hash_key(key);

        if let Some((_, prop)) = self.table.find_mut(hash, |(k, _)| k == key) {
            prop.enumerable = enumerable;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::DynamicMap;

    #[test]
    fn test_dynamic_map() {
        let mut map: DynamicMap<&'static str, i32> = DynamicMap::new();
        map.insert("a", 0);
        map.insert("b", 0);
        map.insert("c", 0);
        map.insert("d", 0);
        let mut current = 0;
        while let Some(next) = map.next(current) {
            if current == 2 {
                map.insert("e", 0);
                map.insert("f", 0);
            }
            println!("{}", map.key_at(current).unwrap());
            current = next;
        }
        println!("next");
        current = 0;
        while let Some(next) = map.next(current) {
            println!("{}", map.key_at(current).unwrap());
            current = next;
        }
    }

    /// The keys JSON.stringify lists: it reads `key_at(i)` for i = 1, 2, ... without `next()`.
    fn keys_by_position(map: &DynamicMap<&'static str, i32>) -> Vec<&'static str> {
        (1..).map_while(|i| map.key_at(i).copied()).collect()
    }

    /// The keys a for-in or AMF lists: `next()`, then `key_at` of what it returned.
    fn keys_by_next(map: &DynamicMap<&'static str, i32>) -> Vec<&'static str> {
        let mut keys = vec![];
        let mut index = map.next(0);
        while let Some(i) = index {
            keys.push(*map.key_at(i).unwrap());
            index = map.next(i);
        }
        keys
    }

    #[test]
    fn listing_by_position_after_a_finished_enumeration_and_an_insert_lists_every_key() {
        // An AQW inventory item's keys. The game's copyObj enumerates it (AMF), then the game adds
        // iQtyNow, then JSON.stringify lists it.
        let keys: Vec<&'static str> = "sIcon bEquip EnhID sFile bUpg iCost iType tPurchase CharID sES \
             sLink iDPS bStaff iHrs sElmt sName sType dPurchase iRng iLvl bCoins iQty iStk bBank sDesc \
             iRty bTemp ItemID CharItemID"
            .split_whitespace()
            .collect();
        for added in ["iQtyNow", "zz", "a", "bExtra2", "bHouse"] {
            let mut map: DynamicMap<&'static str, i32> = DynamicMap::new();
            for key in &keys {
                map.insert(key, 0);
            }

            assert_eq!(keys_by_next(&map).len(), keys.len());
            map.insert(added, 0);

            let mut listed = keys_by_position(&map);
            listed.sort();
            let mut expected: Vec<&str> = keys.iter().copied().chain([added]).collect();
            expected.sort();
            assert_eq!(listed, expected, "after adding {added}");
        }
    }

    #[test]
    fn listing_by_position_after_a_finished_enumeration_and_a_remove_lists_every_key() {
        let mut map: DynamicMap<&'static str, i32> = DynamicMap::new();
        for key in ["a", "b", "c", "d", "e", "f", "g", "h"] {
            map.insert(key, 0);
        }
        let listed = keys_by_next(&map);
        map.remove(&listed[0]);

        let mut after = keys_by_position(&map);
        after.sort();
        let mut expected = listed[1..].to_vec();
        expected.sort();
        assert_eq!(after, expected);
    }
}
