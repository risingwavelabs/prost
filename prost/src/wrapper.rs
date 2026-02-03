//! Transparent wrapper trait and casting utilities.
//!
//! This module provides the `TransparentOver` trait for marking types that are
//! transparent wrappers over an inner type. This allows safe casting between
//! wrapper types and their inner types when used in collections like `Vec`, `HashMap`,
//! and `BTreeMap`.

/// # Safety
///
/// This trait must only be implemented for types that are `#[repr(transparent)]`
/// over the `Inner` type. The memory layout of `Self` must be identical to `Inner`,
/// allowing safe transmutation between `&Self` and `&Inner`, as well as between
/// collections containing these types.
///
/// Implementing this trait incorrectly can lead to undefined behavior.
pub unsafe trait TransparentOver {
    /// The inner type that this wrapper is transparent over.
    type Inner;
}

/// Casts a slice of wrapper types to a slice of inner types.
///
/// # Safety
///
/// This is safe when `T: TransparentOver` with `#[repr(transparent)]` over `T::Inner`.
#[doc(hidden)]
#[doc(hidden)]
pub fn raw_slice<T: TransparentOver>(slice: &[T]) -> &[T::Inner] {
    // SAFETY: T is #[repr(transparent)] over T::Inner, so &[T] and &[T::Inner] have
    // identical memory layout
    unsafe { core::slice::from_raw_parts(slice.as_ptr() as *const T::Inner, slice.len()) }
}

/// Casts a reference to a Vec of wrapper types to a reference to a Vec of inner types.
///
/// # Safety
///
/// This is safe when `T: TransparentOver` with `#[repr(transparent)]` over `T::Inner`.
#[doc(hidden)]
pub fn raw_vec_ref<T: TransparentOver>(vec: &alloc::vec::Vec<T>) -> &alloc::vec::Vec<T::Inner> {
    // SAFETY: T is #[repr(transparent)] over T::Inner, so Vec<T> and Vec<T::Inner> have
    // identical memory layout
    unsafe { &*(vec as *const alloc::vec::Vec<T> as *const alloc::vec::Vec<T::Inner>) }
}

/// Casts a mutable reference to a Vec of wrapper types to a mutable reference to a Vec of inner types.
///
/// # Safety
///
/// This is safe when `T: TransparentOver` with `#[repr(transparent)]` over `T::Inner`.
pub fn mut_raw_vec<T: TransparentOver>(
    vec: &mut alloc::vec::Vec<T>,
) -> &mut alloc::vec::Vec<T::Inner> {
    // SAFETY: T is #[repr(transparent)] over T::Inner, so Vec<T> and Vec<T::Inner> have
    // identical memory layout
    unsafe { &mut *(vec as *mut alloc::vec::Vec<T> as *mut alloc::vec::Vec<T::Inner>) }
}

/// Casts a reference to a HashMap with wrapper keys to a reference to a HashMap with inner keys.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` with `#[repr(transparent)]` over `K::Inner`.
#[cfg(feature = "std")]
#[doc(hidden)]
pub fn raw_hash_map_key_ref<K, V>(
    map: &std::collections::HashMap<K, V>,
) -> &std::collections::HashMap<K::Inner, V>
where
    K: TransparentOver,
{
    // SAFETY: K is #[repr(transparent)] over K::Inner, so HashMap<K, V> and
    // HashMap<K::Inner, V> have identical memory layout
    unsafe {
        &*(map as *const std::collections::HashMap<K, V>
            as *const std::collections::HashMap<K::Inner, V>)
    }
}

/// Casts a mutable reference to a HashMap with wrapper keys to a mutable reference to a HashMap with inner keys.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` with `#[repr(transparent)]` over `K::Inner`.
#[cfg(feature = "std")]
#[doc(hidden)]
pub fn raw_hash_map_key_mut_ref<K, V>(
    map: &mut std::collections::HashMap<K, V>,
) -> &mut std::collections::HashMap<K::Inner, V>
where
    K: TransparentOver,
{
    // SAFETY: K is #[repr(transparent)] over K::Inner, so HashMap<K, V> and
    // HashMap<K::Inner, V> have identical memory layout
    unsafe {
        &mut *(map as *mut std::collections::HashMap<K, V>
            as *mut std::collections::HashMap<K::Inner, V>)
    }
}

/// Casts a reference to a HashMap with wrapper values to a reference to a HashMap with inner values.
///
/// # Safety
///
/// This is safe when `V: TransparentOver` with `#[repr(transparent)]` over `V::Inner`.
#[cfg(feature = "std")]
#[doc(hidden)]
pub fn raw_hash_map_value_ref<K, V>(
    map: &std::collections::HashMap<K, V>,
) -> &std::collections::HashMap<K, V::Inner>
where
    V: TransparentOver,
{
    // SAFETY: V is #[repr(transparent)] over V::Inner, so HashMap<K, V> and
    // HashMap<K, V::Inner> have identical memory layout
    unsafe {
        &*(map as *const std::collections::HashMap<K, V>
            as *const std::collections::HashMap<K, V::Inner>)
    }
}

/// Casts a mutable reference to a HashMap with wrapper values to a mutable reference to a HashMap with inner values.
///
/// # Safety
///
/// This is safe when `V: TransparentOver` with `#[repr(transparent)]` over `V::Inner`.
#[cfg(feature = "std")]
#[doc(hidden)]
pub fn raw_hash_map_value_mut_ref<K, V>(
    map: &mut std::collections::HashMap<K, V>,
) -> &mut std::collections::HashMap<K, V::Inner>
where
    V: TransparentOver,
{
    // SAFETY: V is #[repr(transparent)] over V::Inner, so HashMap<K, V> and
    // HashMap<K, V::Inner> have identical memory layout
    unsafe {
        &mut *(map as *mut std::collections::HashMap<K, V>
            as *mut std::collections::HashMap<K, V::Inner>)
    }
}

/// Casts a reference to a HashMap with both wrapper keys and values to a reference to a HashMap with inner types.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` and `V: TransparentOver` with `#[repr(transparent)]`.
#[cfg(feature = "std")]
#[doc(hidden)]
pub fn raw_hash_map_ref<K, V>(
    map: &std::collections::HashMap<K, V>,
) -> &std::collections::HashMap<K::Inner, V::Inner>
where
    K: TransparentOver,
    V: TransparentOver,
{
    // SAFETY: K and V are #[repr(transparent)] over their Inner types, so
    // HashMap<K, V> and HashMap<K::Inner, V::Inner> have identical memory layout
    unsafe {
        &*(map as *const std::collections::HashMap<K, V>
            as *const std::collections::HashMap<K::Inner, V::Inner>)
    }
}

/// Casts a mutable reference to a HashMap with both wrapper keys and values to a mutable reference to a HashMap with inner types.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` and `V: TransparentOver` with `#[repr(transparent)]`.
#[cfg(feature = "std")]
#[doc(hidden)]
pub fn raw_hash_map_mut_ref<K, V>(
    map: &mut std::collections::HashMap<K, V>,
) -> &mut std::collections::HashMap<K::Inner, V::Inner>
where
    K: TransparentOver,
    V: TransparentOver,
{
    // SAFETY: K and V are #[repr(transparent)] over their Inner types, so
    // HashMap<K, V> and HashMap<K::Inner, V::Inner> have identical memory layout
    unsafe {
        &mut *(map as *mut std::collections::HashMap<K, V>
            as *mut std::collections::HashMap<K::Inner, V::Inner>)
    }
}

/// Casts a reference to a BTreeMap with wrapper keys to a reference to a BTreeMap with inner keys.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` with `#[repr(transparent)]` over `K::Inner`.
#[doc(hidden)]
pub fn raw_btree_map_key_ref<K, V>(
    map: &alloc::collections::BTreeMap<K, V>,
) -> &alloc::collections::BTreeMap<K::Inner, V>
where
    K: TransparentOver,
{
    // SAFETY: K is #[repr(transparent)] over K::Inner, so BTreeMap<K, V> and
    // BTreeMap<K::Inner, V> have identical memory layout
    unsafe {
        &*(map as *const alloc::collections::BTreeMap<K, V>
            as *const alloc::collections::BTreeMap<K::Inner, V>)
    }
}

/// Casts a mutable reference to a BTreeMap with wrapper keys to a mutable reference to a BTreeMap with inner keys.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` with `#[repr(transparent)]` over `K::Inner`.
#[doc(hidden)]
pub fn raw_btree_map_key_mut_ref<K, V>(
    map: &mut alloc::collections::BTreeMap<K, V>,
) -> &mut alloc::collections::BTreeMap<K::Inner, V>
where
    K: TransparentOver,
{
    // SAFETY: K is #[repr(transparent)] over K::Inner, so BTreeMap<K, V> and
    // BTreeMap<K::Inner, V> have identical memory layout
    unsafe {
        &mut *(map as *mut alloc::collections::BTreeMap<K, V>
            as *mut alloc::collections::BTreeMap<K::Inner, V>)
    }
}

/// Casts a reference to a BTreeMap with wrapper values to a reference to a BTreeMap with inner values.
///
/// # Safety
///
/// This is safe when `V: TransparentOver` with `#[repr(transparent)]` over `V::Inner`.
#[doc(hidden)]
pub fn raw_btree_map_value_ref<K, V>(
    map: &alloc::collections::BTreeMap<K, V>,
) -> &alloc::collections::BTreeMap<K, V::Inner>
where
    V: TransparentOver,
{
    // SAFETY: V is #[repr(transparent)] over V::Inner, so BTreeMap<K, V> and
    // BTreeMap<K, V::Inner> have identical memory layout
    unsafe {
        &*(map as *const alloc::collections::BTreeMap<K, V>
            as *const alloc::collections::BTreeMap<K, V::Inner>)
    }
}

/// Casts a mutable reference to a BTreeMap with wrapper values to a mutable reference to a BTreeMap with inner values.
///
/// # Safety
///
/// This is safe when `V: TransparentOver` with `#[repr(transparent)]` over `V::Inner`.
#[doc(hidden)]
pub fn raw_btree_map_value_mut_ref<K, V>(
    map: &mut alloc::collections::BTreeMap<K, V>,
) -> &mut alloc::collections::BTreeMap<K, V::Inner>
where
    V: TransparentOver,
{
    // SAFETY: V is #[repr(transparent)] over V::Inner, so BTreeMap<K, V> and
    // BTreeMap<K, V::Inner> have identical memory layout
    unsafe {
        &mut *(map as *mut alloc::collections::BTreeMap<K, V>
            as *mut alloc::collections::BTreeMap<K, V::Inner>)
    }
}

/// Casts a reference to a BTreeMap with both wrapper keys and values to a reference to a BTreeMap with inner types.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` and `V: TransparentOver` with `#[repr(transparent)]`.
#[doc(hidden)]
pub fn raw_btree_map_ref<K, V>(
    map: &alloc::collections::BTreeMap<K, V>,
) -> &alloc::collections::BTreeMap<K::Inner, V::Inner>
where
    K: TransparentOver,
    V: TransparentOver,
{
    // SAFETY: K and V are #[repr(transparent)] over their Inner types, so
    // BTreeMap<K, V> and BTreeMap<K::Inner, V::Inner> have identical memory layout
    unsafe {
        &*(map as *const alloc::collections::BTreeMap<K, V>
            as *const alloc::collections::BTreeMap<K::Inner, V::Inner>)
    }
}

/// Casts a mutable reference to a BTreeMap with both wrapper keys and values to a mutable reference to a BTreeMap with inner types.
///
/// # Safety
///
/// This is safe when `K: TransparentOver` and `V: TransparentOver` with `#[repr(transparent)]`.
#[doc(hidden)]
pub fn raw_btree_map_mut_ref<K, V>(
    map: &mut alloc::collections::BTreeMap<K, V>,
) -> &mut alloc::collections::BTreeMap<K::Inner, V::Inner>
where
    K: TransparentOver,
    V: TransparentOver,
{
    // SAFETY: K and V are #[repr(transparent)] over their Inner types, so
    // BTreeMap<K, V> and BTreeMap<K::Inner, V::Inner> have identical memory layout
    unsafe {
        &mut *(map as *mut alloc::collections::BTreeMap<K, V>
            as *mut alloc::collections::BTreeMap<K::Inner, V::Inner>)
    }
}
