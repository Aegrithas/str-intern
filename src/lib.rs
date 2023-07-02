#![cfg_attr(docs_rs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod sync;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::{Iter as SetIter, IntoIter as SetIntoIter};
use std::fmt::{self, Debug, Formatter};
use std::hash::BuildHasher;
use std::iter::{Sum, Product, FusedIterator};
use std::rc::Rc;

/**
 * The type of strings that have been interned.
 * 
 * Currently just a type alias, but I might change that if I find a good reason.
 */
pub type InternedStr = Rc<str>;

/**
 * An interner will keep track of strings and ensure there is only one allocation for any given string contents.
 * 
 * For example:
 * ```rust
 * # use str_intern::{Interner, InternedStr};
 * let mut interner = Interner::new();
 * let foo0 = interner.intern(String::from("foo"));
 * let foo1 = interner.intern(String::from("foo"));
 * assert!(InternedStr::ptr_eq(&foo0, &foo1));
 * ```
 * Because `foo0` and `foo1` have the same contents, they become a single allocation.
 * 
 * Interned strings are immutable, which means that you must construct the finished string before interning it.
 * 
 * This is useful if you have many instances of the same strings
 * (e.g., if 200 different structs contain the string `"foo"`, an interner allows there to be 200 pointers to one allocation, rather than 200 different allocations).
 * 
 * This `Interner` is not thread-safe (which is to say, it is implements neither [`Send`] nor [`Sync`]). For a thread-safe variant, see the [`sync`](crate::sync) module.
 */
#[repr(transparent)]
pub struct Interner<S = RandomState> {
  
  strings: HashSet<InternedStr, S>
  
}

impl Interner {
  
  /**
   * Constructs a new `Interner`.
   */
  pub fn new() -> Self {
    Self::from_set(HashSet::new())
  }
  
}

impl<S> Interner<S> {
  
  /**
   * Constructs a new `Interner` with the given hasher. See [`BuildHasher`] for more information.
   */
  pub fn with_hasher(hasher: S) -> Self {
    Self::from_set(HashSet::with_hasher(hasher))
  }
  
  /**
   * Construct a new `Interner` with the given set's contents already interned.
   * The new `Interner` will also use the given set's hasher.
   */
  pub fn from_set(strings: HashSet<InternedStr, S>) -> Self {
    Self { strings }
  }
  
  /**
   * Consume this `Interner` and return a set containing all of strings that were interned.
   * The returned set also uses the same hasher.
   */
  pub fn into_set(self) -> HashSet<InternedStr, S> {
    self.strings
  }
  
  /**
   * Removes all of the interned strings.
   */
  pub fn clear(&mut self) {
    self.strings.clear();
  }
  
  /**
   * An iterator over all of the currently interned strings.
   */
  pub fn iter(&self) -> Iter {
    Iter::new(self.strings.iter())
  }
  
}

impl<S: BuildHasher> Interner<S> {
  
  /**
   * Saves the given string if it is not already saved, and returns the saved string.
   */
  pub fn intern(&mut self, string: impl AsRef<str>) -> InternedStr {
    // Sorrow abounds, for behold: HashSet::get_or_insert_with doesn't exist yet.
    let string = string.as_ref();
    match self.strings.get(string) {
      Some(string) => string.clone(),
      None => {
        let string = InternedStr::from(string);
        self.strings.insert(InternedStr::clone(&string));
        string
      }
    }
  }
  
}

impl<S: Clone> Clone for Interner<S> {
  
  fn clone(&self) -> Self {
    Interner { strings: self.strings.clone() }
  }
  
  fn clone_from(&mut self, source: &Self) {
    self.strings.clone_from(&source.strings)
  }
  
}

impl<S: BuildHasher> PartialEq for Interner<S> {
  
  fn eq(&self, other: &Self) -> bool {
    self.strings.eq(&other.strings)
  }
  
  fn ne(&self, other: &Self) -> bool {
    self.strings.ne(&other.strings)
  }
  
}

impl<S: BuildHasher> Eq for Interner<S> {}

impl<S> Debug for Interner<S> {
  
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("Interner").field(&self.strings).finish()
  }
  
}

impl<S: Default> Default for Interner<S> {
  
  fn default() -> Self {
    Self { strings: HashSet::default() }
  }
  
}

impl<S> IntoIterator for Interner<S> {
  
  type Item = InternedStr;
  type IntoIter = IntoIter;
  
  fn into_iter(self) -> IntoIter {
    IntoIter::new(self.strings.into_iter())
  }
  
}

impl<'a, S> IntoIterator for &'a Interner<S> {
  
  type Item = &'a InternedStr;
  type IntoIter = Iter<'a>;
  
  fn into_iter(self) -> Iter<'a> {
    Iter::new(self.strings.iter())
  }
  
}

impl<A, S> FromIterator<A> for Interner<S> where HashSet<InternedStr, S>: FromIterator<A> {
  
  fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
    Self::from_set(HashSet::from_iter(iter))
  }
  
}

/**
 * An iterator over the strings in an `Interner`.
 * 
 * This `struct` is created by the [`iter`](Interner::iter) method on `Interner`.
 */
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Iter<'a> {
  
  iter: SetIter<'a, InternedStr>
  
}

impl<'a> Iter<'a> {
  
  fn new(iter: SetIter<'a, InternedStr>) -> Self {
    Self { iter }
  }
  
}

impl<'a> Iterator for Iter<'a> {
  
  type Item =  &'a InternedStr;
  
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
  
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
  
  fn count(self) -> usize {
    self.iter.count()
  }
  
  fn last(self) -> Option<Self::Item> {
    self.iter.last()
  }
  
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    self.iter.nth(n)
  }
  
  fn for_each<F: FnMut(Self::Item)>(self, f: F) {
    self.iter.for_each(f)
  }
  
  fn collect<B: FromIterator<Self::Item>>(self) -> B {
    self.iter.collect()
  }
  
  fn partition<B: Default + Extend<Self::Item>, F: FnMut(&Self::Item) -> bool>(self, f: F) -> (B, B) {
    self.iter.partition(f)
  }
  
  fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, f: F) -> B {
    self.iter.fold(init, f)
  }
  
  fn reduce<F: FnMut(Self::Item, Self::Item) -> Self::Item>(self, f: F) -> Option<Self::Item> {
    self.iter.reduce(f)
  }
  
  fn all<F: FnMut(Self::Item) -> bool>(&mut self, f: F) -> bool {
    self.iter.all(f)
  }
  
  fn any<F: FnMut(Self::Item) -> bool>(&mut self, f: F) -> bool {
    self.iter.any(f)
  }
  
  fn find<P: FnMut(&Self::Item) -> bool>(&mut self, predicate: P) -> Option<Self::Item> {
    self.iter.find(predicate)
  }
  
  fn find_map<B, F: FnMut(Self::Item) -> Option<B>>(&mut self, f: F) -> Option<B> {
    self.iter.find_map(f)
  }
  
  fn position<P: FnMut(Self::Item) -> bool>(&mut self, predicate: P) -> Option<usize> {
    self.iter.position(predicate)
  }
  
  fn max(self) -> Option<Self::Item> where Self::Item: Ord {
    self.iter.max()
  }
  
  fn min(self) -> Option<Self::Item> where Self::Item: Ord {
    self.iter.min()
  }
  
  fn max_by_key<B: Ord, F: FnMut(&Self::Item) -> B>(self, f: F) -> Option<Self::Item> {
    self.iter.max_by_key(f)
  }
  
  fn max_by<F: FnMut(&Self::Item, &Self::Item) -> Ordering>(self, compare: F) -> Option<Self::Item> {
    self.iter.max_by(compare)
  }
  
  fn min_by_key<B: Ord, F: FnMut(&Self::Item) -> B>(self, f: F) -> Option<Self::Item> {
    self.iter.min_by_key(f)
  }
  
  fn min_by<F: FnMut(&Self::Item, &Self::Item) -> Ordering>(self, compare: F) -> Option<Self::Item> {
    self.iter.min_by(compare)
  }
  
  fn sum<S: Sum<Self::Item>>(self) -> S {
    self.iter.sum()
  }
  
  fn product<P: Product<Self::Item>>(self) -> P {
    self.iter.product()
  }
  
  fn cmp<I: IntoIterator<Item = Self::Item>>(self, other: I) -> Ordering where Self::Item: Ord {
    self.iter.cmp(other)
  }
  
  fn partial_cmp<I: IntoIterator>(self, other: I) -> Option<Ordering> where Self::Item: PartialOrd<I::Item> {
    self.iter.partial_cmp(other)
  }
  
  fn eq<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialEq<I::Item> {
    self.iter.eq(other)
  }
  
  fn ne<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialEq<I::Item> {
    self.iter.ne(other)
  }
  
  fn lt<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.lt(other)
  }
  
  fn le<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.le(other)
  }
  
  fn gt<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.gt(other)
  }
  
  fn ge<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.ge(other)
  }
  
}

impl<'a> ExactSizeIterator for Iter<'a> {
  
  fn len(&self) -> usize {
    self.iter.len()
  }
  
}

impl<'a> FusedIterator for Iter<'a> {}

/**
 * An owning iterator over the strings that were in an `Interner`.
 * 
 * This `struct` is created by the [`into_iter`](IntoIterator::into_iter) method on [`Interner`]
 * (provided by the [`IntoIterator`] trait).
 */
#[repr(transparent)]
#[derive(Debug)]
pub struct IntoIter {
  
  iter: SetIntoIter<InternedStr>
  
}

impl IntoIter {
  
  fn new(iter: SetIntoIter<InternedStr>) -> Self {
    Self { iter }
  }
  
}

impl Iterator for IntoIter {
  
  type Item = InternedStr;
  
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next()
  }
  
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }
  
  fn count(self) -> usize {
    self.iter.count()
  }
  
  fn last(self) -> Option<Self::Item> {
    self.iter.last()
  }
  
  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    self.iter.nth(n)
  }
  
  fn for_each<F: FnMut(Self::Item)>(self, f: F) {
    self.iter.for_each(f)
  }
  
  fn collect<B: FromIterator<Self::Item>>(self) -> B {
    self.iter.collect()
  }
  
  fn partition<B: Default + Extend<Self::Item>, F: FnMut(&Self::Item) -> bool>(self, f: F) -> (B, B) {
    self.iter.partition(f)
  }
  
  fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, f: F) -> B {
    self.iter.fold(init, f)
  }
  
  fn reduce<F: FnMut(Self::Item, Self::Item) -> Self::Item>(self, f: F) -> Option<Self::Item> {
    self.iter.reduce(f)
  }
  
  fn all<F: FnMut(Self::Item) -> bool>(&mut self, f: F) -> bool {
    self.iter.all(f)
  }
  
  fn any<F: FnMut(Self::Item) -> bool>(&mut self, f: F) -> bool {
    self.iter.any(f)
  }
  
  fn find<P: FnMut(&Self::Item) -> bool>(&mut self, predicate: P) -> Option<Self::Item> {
    self.iter.find(predicate)
  }
  
  fn find_map<B, F: FnMut(Self::Item) -> Option<B>>(&mut self, f: F) -> Option<B> {
    self.iter.find_map(f)
  }
  
  fn position<P: FnMut(Self::Item) -> bool>(&mut self, predicate: P) -> Option<usize> {
    self.iter.position(predicate)
  }
  
  fn max(self) -> Option<Self::Item> where Self::Item: Ord {
    self.iter.max()
  }
  
  fn min(self) -> Option<Self::Item> where Self::Item: Ord {
    self.iter.min()
  }
  
  fn max_by_key<B: Ord, F: FnMut(&Self::Item) -> B>(self, f: F) -> Option<Self::Item> {
    self.iter.max_by_key(f)
  }
  
  fn max_by<F: FnMut(&Self::Item, &Self::Item) -> Ordering>(self, compare: F) -> Option<Self::Item> {
    self.iter.max_by(compare)
  }
  
  fn min_by_key<B: Ord, F: FnMut(&Self::Item) -> B>(self, f: F) -> Option<Self::Item> {
    self.iter.min_by_key(f)
  }
  
  fn min_by<F: FnMut(&Self::Item, &Self::Item) -> Ordering>(self, compare: F) -> Option<Self::Item> {
    self.iter.min_by(compare)
  }
  
  fn sum<S: Sum<Self::Item>>(self) -> S {
    self.iter.sum()
  }
  
  fn product<P: Product<Self::Item>>(self) -> P {
    self.iter.product()
  }
  
  fn cmp<I: IntoIterator<Item = Self::Item>>(self, other: I) -> Ordering where Self::Item: Ord {
    self.iter.cmp(other)
  }
  
  fn partial_cmp<I: IntoIterator>(self, other: I) -> Option<Ordering> where Self::Item: PartialOrd<I::Item> {
    self.iter.partial_cmp(other)
  }
  
  fn eq<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialEq<I::Item> {
    self.iter.eq(other)
  }
  
  fn ne<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialEq<I::Item> {
    self.iter.ne(other)
  }
  
  fn lt<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.lt(other)
  }
  
  fn le<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.le(other)
  }
  
  fn gt<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.gt(other)
  }
  
  fn ge<I: IntoIterator>(self, other: I) -> bool where Self::Item: PartialOrd<I::Item> {
    self.iter.ge(other)
  }
  
}

impl ExactSizeIterator for IntoIter {
  
  fn len(&self) -> usize {
    self.iter.len()
  }
  
}

impl FusedIterator for IntoIter {}