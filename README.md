# Hitung

Hitung is a very simple programming language for calculation.

## Motivation

Mostly for Indonesian who want to learn how programming language works. You can read the tutorial for this on my blog here in Bahasa Indonesia
1. 
2.
3.


## Process
1. Lexical Analysis
2. Parsing
3. JIT (just in time) compiler with LLVM

## What this language can do

example:

- Variable assignment
```rust
a = 8
b = a + 2 * 3
```
this will print `14`

- As calculator
```rust
2 + 5 * 3 / 3 * 7 - 10
```
this will evaluate to `2 + (((5 * 3) / 3) * 7 ) - 10` and this will print `27` for the result

- Conditional (only support 1 level)
```rust
if 1 > 2 then 1 else 0
```
this will print `0`

## License
MIT @Aldi Priya Perdana