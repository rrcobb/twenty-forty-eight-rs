# 2048 game in rust

- using https://lib.rs/crates/pixels

## row coalesce right

x[None, None, None, None] => [None, None, None, None]
x[None, None, None, Some(1)] => [None, None, None, Some(1)]
x[None, None, Some(1), None] => [None, None, None, Some(1)]
x[None, None, Some(1), Some(1)] => [None, None, None, Some(2)]
x[None, Some(1), Some(1), Some(1)] => [None, None, Some(1), Some(2)]
x[Some(1), Some(1), Some(1), Some(1)] => [None, None, Some(2), Some(2)]
[Some(1), Some(1), Some(1), None] => [None, None, Some(1), Some(2)]
[Some(1), None, Some(1), Some(1)] => [None, None, Some(1), Some(2)]
[Some(1), None, Some(1), Some(2)] => [None, None, Some(2), Some(2)]
[Some(1), None, Some(2), Some(1)] => [None, Some(1), Some(2), Some(1)]
x[None, Some(2), Some(1), Some(1)] => [None, None, Some(2), Some(2)]

for each item, from end to start
  shift right until you find a wall, or an item, then try to merge
    can merge if 1) same values, 2) hasn't been merged already this coalesce
      [None, Some(2), Some(1), Some(1)] => [None, None, Some(2), Some(2)] is the
      edge case to worry over

