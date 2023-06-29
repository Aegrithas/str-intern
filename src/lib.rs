pub mod sync;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::{Iter as SetIter, IntoIter as SetIntoIter};
use std::fmt::{self, Debug, Formatter};
use std::hash::BuildHasher;
use std::iter::{Sum, Product, FusedIterator};
use std::rc::Rc;

pub type InternedStr = Rc<str>;

#[repr(transparent)]
pub struct Interner<S = RandomState> {
  
  strings: HashSet<Rc<str>, S>
  
}

impl Interner {
  
  pub fn new() -> Self {
    Self::from_set(HashSet::new())
  }
  
}

impl<S> Interner<S> {
  
  pub fn with_hasher(hasher: S) -> Self {
    Self::from_set(HashSet::with_hasher(hasher))
  }
  
  pub fn from_set(strings: HashSet<Rc<str>, S>) -> Self {
    Self { strings }
  }
  
  pub fn into_set(self) -> HashSet<Rc<str>, S> {
    self.strings
  }
  
  pub fn intern(&mut self, string: impl AsRef<str>) -> Rc<str> where S: BuildHasher {
    // Sorrow abounds, for behold: HashSet::get_or_insert_with doesn't exist yet.
    let string = string.as_ref();
    match self.strings.get(string) {
      Some(string) => string.clone(),
      None => {
        let string = Rc::from(string);
        self.strings.insert(Rc::clone(&string));
        string
      }
    }
  }
  
  pub fn clear(&mut self) {
    self.strings.clear();
  }
  
  pub fn iter(&self) -> Iter {
    Iter::new(self.strings.iter())
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
  
  type Item = Rc<str>;
  type IntoIter = IntoIter;
  
  fn into_iter(self) -> IntoIter {
    IntoIter::new(self.strings.into_iter())
  }
  
}

impl<'a, S> IntoIterator for &'a Interner<S> {
  
  type Item = &'a Rc<str>;
  type IntoIter = Iter<'a>;
  
  fn into_iter(self) -> Iter<'a> {
    Iter::new(self.strings.iter())
  }
  
}

impl<A, S> FromIterator<A> for Interner<S> where HashSet<Rc<str>, S>: FromIterator<A> {
  
  fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
    Self::from_set(HashSet::from_iter(iter))
  }
  
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Iter<'a> {
  
  iter: SetIter<'a, Rc<str>>
  
}

impl<'a> Iter<'a> {
  
  fn new(iter: SetIter<'a, Rc<str>>) -> Self {
    Self { iter }
  }
  
}

impl<'a> Iterator for Iter<'a> {
  
  type Item =  &'a Rc<str>;
  
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

#[repr(transparent)]
#[derive(Debug)]
pub struct IntoIter {
  
  iter: SetIntoIter<Rc<str>>
  
}

impl IntoIter {
  
  fn new(iter: SetIntoIter<Rc<str>>) -> Self {
    Self { iter }
  }
  
}

impl Iterator for IntoIter {
  
  type Item = Rc<str>;
  
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