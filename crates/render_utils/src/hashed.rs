use ahash::AHasher;
use std::{
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

// /// A pre-hashed value of a specific type. Pre-hashing enables memoization of hashes that are expensive to compute.
// /// It also enables faster [`PartialEq`] comparisons by short circuiting on hash equality.
// /// See [`PassHash`] and [`PassHasher`] for a "pass through" [`BuildHasher`] and [`Hasher`] implementation
// /// designed to work with [`Hashed`]
// /// See [`PreHashMap`] for a hashmap pre-configured to use [`Hashed`] keys.
// pub struct Hashed<V, H = FixedState> {
//     hash: u64,
//     value: V,
//     marker: PhantomData<H>,
// }

// impl<V: Hash, H: BuildHasher + Default> Hashed<V, H> {
//     /// Pre-hashes the given value using the [`BuildHasher`] configured in the [`Hashed`] type.
//     pub fn new(value: V) -> Self {
//         let builder = H::default();
//         let mut hasher = builder.build_hasher();
//         value.hash(&mut hasher);
//         Self {
//             hash: hasher.finish(),
//             value,
//             marker: PhantomData,
//         }
//     }

//     /// The pre-computed hash.
//     #[inline]
//     pub fn hash(&self) -> u64 {
//         self.hash
//     }
// }

// impl<V, H> Hash for Hashed<V, H> {
//     #[inline]
//     fn hash<R: Hasher>(&self, state: &mut R) {
//         state.write_u64(self.hash);
//     }
// }

// impl<V, H> Deref for Hashed<V, H> {
//     type Target = V;

//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }

// impl<V: PartialEq, H> PartialEq for Hashed<V, H> {
//     /// A fast impl of [`PartialEq`] that first checks that `other`'s pre-computed hash
//     /// matches this value's pre-computed hash.
//     #[inline]
//     fn eq(&self, other: &Self) -> bool {
//         self.hash == other.hash && self.value.eq(&other.value)
//     }
// }

// impl<V: Debug, H> Debug for Hashed<V, H> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Hashed")
//             .field("hash", &self.hash)
//             .field("value", &self.value)
//             .finish()
//     }
// }

// impl<V: Clone, H> Clone for Hashed<V, H> {
//     #[inline]
//     fn clone(&self) -> Self {
//         Self {
//             hash: self.hash,
//             value: self.value.clone(),
//             marker: PhantomData,
//         }
//     }
// }

// impl<V: Eq, H> Eq for Hashed<V, H> {}

// /// A hasher builder that will create a fixed hasher.
// #[derive(Debug, Clone, Default)]
// pub struct FixedState;

// impl std::hash::BuildHasher for FixedState {
//     type Hasher = AHasher;

//     #[inline]
//     fn build_hasher(&self) -> AHasher {
//         AHasher::new_with_keys(
//             0b1001010111101110000001001100010000000011001001101011001001111000,
//             0b1100111101101011011110001011010100000100001111100011010011010101,
//         )
//     }
// }
