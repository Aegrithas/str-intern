use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::{Iter as SetIter, IntoIter as SetIntoIter};
use std::fmt::{self, Debug, Formatter};
use std::hash::BuildHasher;
use std::iter::{Sum, Product, FusedIterator};
#[cfg(feature = "global")]
use std::ops::Deref;
use std::sync::{Arc, OnceLock, Mutex, MutexGuard};

pub type InternedStr = Arc<str>;

#[repr(transparent)]
pub struct Interner<S = RandomState> {
  
  strings: Mutex<HashSet<Arc<str>, S>>
  
}

impl Interner {
  
  pub fn new() -> Self {
    Self::from_set(HashSet::new())
  }
  
}

impl<S> Interner<S> {
  
  const POISON_MESSAGE: &'static str = "Interner mutex was poisoned";
  
  pub fn with_hasher(hasher: S) -> Self {
    Self::from_set(HashSet::with_hasher(hasher))
  }
  
  pub fn from_set(strings: HashSet<Arc<str>, S>) -> Self {
    Self { strings: Mutex::new(strings) }
  }
  
  pub fn into_set(self) -> HashSet<Arc<str>, S> {
    self.strings.into_inner().expect(Self::POISON_MESSAGE)
  }
  
  fn strings(&self) -> MutexGuard<HashSet<Arc<str>, S>> {
    self.strings.lock().expect(Self::POISON_MESSAGE)
  }
  
  pub fn intern(&self, string: impl AsRef<str>) -> Arc<str> where S: BuildHasher {
    self.lock().intern(string)
  }
  
  pub fn clear(&self) {
    self.strings().clear();
  }
  
  pub fn lock(&self) -> LockedInterner<S> {
    LockedInterner::new(self.strings())
  }
  
}

impl<S: Clone> Clone for Interner<S> {
  
  fn clone(&self) -> Self {
    Interner { strings: Mutex::new(self.strings().clone()) }
  }
  
  fn clone_from(&mut self, source: &Self) {
    self.strings().clone_from(&source.strings())
  }
  
}

impl<S: BuildHasher> PartialEq for Interner<S> {
  
  fn eq(&self, other: &Self) -> bool {
    self.strings().eq(&other.strings())
  }
  
  fn ne(&self, other: &Self) -> bool {
    self.strings().ne(&other.strings())
  }
  
}

impl<S: BuildHasher> Eq for Interner<S> {}

impl<S> Debug for Interner<S> {
  
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("Interner").field(&self.strings()).finish()
  }
  
}

impl<S: Default> Default for Interner<S> {
  
  fn default() -> Self {
    Self { strings: Mutex::default() }
  }
  
}

impl<S> IntoIterator for Interner<S> {
  
  type Item = Arc<str>;
  type IntoIter = IntoIter;
  
  fn into_iter(self) -> IntoIter {
    IntoIter::new(self.into_set().into_iter())
  }
  
}

impl<A, S> FromIterator<A> for Interner<S> where HashSet<Arc<str>, S>: FromIterator<A> {
  
  fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
    Self::from_set(HashSet::from_iter(iter))
  }
  
}

#[repr(transparent)]
pub struct LockedInterner<'a, S = RandomState> {
  
  strings: MutexGuard<'a, HashSet<Arc<str>, S>>
  
}

impl<'a, S> LockedInterner<'a, S> {
  
  fn new(strings: MutexGuard<'a, HashSet<Arc<str>, S>>) -> Self {
    Self { strings }
  }
  
  pub fn intern(&mut self, string: impl AsRef<str>) -> Arc<str> where S: BuildHasher {
    // Sorrow abounds, for behold: HashSet::get_or_insert_with doesn't exist yet.
    let string = string.as_ref();
    match self.strings.get(string) {
      Some(string) => string.clone(),
      None => {
        let string = Arc::from(string);
        self.strings.insert(Arc::clone(&string));
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

impl<'a, S: BuildHasher> PartialEq for LockedInterner<'a, S> {
  
  fn eq(&self, other: &Self) -> bool {
    self.strings.eq(&other.strings)
  }
  
  fn ne(&self, other: &Self) -> bool {
    self.strings.ne(&other.strings)
  }
  
}

impl<'a, S: BuildHasher> Eq for LockedInterner<'a, S> {}

impl<'a, S> Debug for LockedInterner<'a, S> {
  
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_tuple("Interner").field(&self.strings).finish()
  }
  
}

impl<'a, 'b, S> IntoIterator for &'b LockedInterner<'a, S> {
  
  type Item = &'b Arc<str>;
  type IntoIter = Iter<'b>;
  
  fn into_iter(self) -> Iter<'b> {
    Iter::new(self.strings.iter())
  }
  
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Iter<'a> {
  
  iter: SetIter<'a, Arc<str>>
  
}

impl<'a> Iter<'a> {
  
  fn new(iter: SetIter<'a, Arc<str>>) -> Self {
    Self { iter }
  }
  
}

impl<'a> Iterator for Iter<'a> {
  
  type Item =  &'a Arc<str>;
  
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
  
  iter: SetIntoIter<Arc<str>>
  
}

impl IntoIter {
  
  fn new(iter: SetIntoIter<Arc<str>>) -> Self {
    Self { iter }
  }
  
}

impl Iterator for IntoIter {
  
  type Item = Arc<str>;
  
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

#[cfg(feature = "global")]
static GLOBAL: OnceLock<Interner> = OnceLock::new();

#[cfg(feature = "global")]
pub struct GlobalInterner;

#[cfg(feature = "global")]
impl Deref for GlobalInterner {
  
  type Target = Interner;
  
  fn deref(&self) -> &Interner {
    GLOBAL.get_or_init(Interner::new)
  }
  
}

#[cfg(feature = "global")]
#[inline]
pub fn intern(string: impl AsRef<str>) -> Arc<str> {
  GlobalInterner.intern(string)
}

#[cfg(feature = "global")]
pub trait InternExt: AsRef<str> {
  
  #[inline]
  fn intern(&self) -> Arc<str> {
    intern(self)
  }
  
}

#[cfg(feature = "global")]
impl InternExt for String {}

#[cfg(feature = "global")]
impl InternExt for str {}

#[cfg(feature = "global")]
impl InternExt for Box<str> {}

#[cfg(feature = "global")]
impl InternExt for Arc<str> {}

#[cfg(feature = "global")]
impl<T: InternExt + ?Sized> InternExt for &'_ T {}

#[cfg(feature = "global")]
impl<T: InternExt + ?Sized> InternExt for &'_ mut T {}