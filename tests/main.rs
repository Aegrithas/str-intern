use std::sync::Arc;

use str_intern::sync::*;

#[test]
fn main() {
  let s0 = intern("Hello World!".to_string());
  let s1 = "Hello World!".intern();
  assert!(Arc::ptr_eq(&s0, &s1));
}