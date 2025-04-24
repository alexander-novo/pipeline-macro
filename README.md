# `pipeline-macro`

The `pipe!` macro allows for nested function evaluations which are written left to right, rather than inside out (and often right to left) similarly to pipes in shells or the pipe operators in C++, Elixir, or F#.

## Usage Examples

A basic usage is given as follows:

```rust
use pipeline_macro::pipe;

fn fn1(x: i32) -> i32 {x+1}
fn fn2(x: i32) -> i32 {2*x}
let x = 5;

assert_eq!(
	pipe!(x+2 => fn1 => fn2),
	fn2(fn1(x+2))
);
```

Functions which take multiple arguments as input are supported - you simply need to specify which arguments to pipe into using the wildcard (`_`):

```rust
use pipeline_macro::pipe;

fn fn1(x: i32) -> i32 {x+1}
fn fn2(x: i32, y:i32) -> (i32, i32) {(x, y)}
let x = 5;
let y = 1;

assert_eq!(
	pipe!(x+2 => fn1 => fn2(y, _)),
	fn2(y, fn1(x+2))
);
```

Note that the output from all previous function evaluations will be inserted into *each* wildcard character, although previous functions will only be executed only once:

```rust
# use pipeline_macro::pipe;
# fn fn1(x: i32) -> i32 {x+1}
# fn fn2(x: i32, y:i32) -> (i32, i32) {(x, y)}
# let x = 5;
# let y = 1;
#
assert_eq!(
	pipe!(x+2 => fn1 => fn2(_, _)),
	fn2(fn1(x+2), fn1(x+2))
);
```

Expressions which can be evaluated like a function (such as lambdas) are also supported:

```rust
use pipeline_macro::pipe;

fn fn2(x: i32, y:i32) -> (i32, i32) {(x, y)}
let x = 5;
let y = 1;

assert_eq!(
	pipe!(x+2 => |x| x-1 => fn2(_, y)),
	fn2((|x| x-1)(x+2), y)
);
```

Note that to support expressions which can be evaluated like a multi-argument function need to be parenthesized before specifying which arguments are piped:

```rust
# #![feature(macro_metavar_expr)]
# use paste::paste;
# 
# // A macro which partially evaluates a function
# macro_rules! bind {
# 	($($f:ident).+ ($($($head:expr_2021),*,)? $(_ $(, $tail:expr_2021)*),*)) => {
# 		paste! {
# 			|$([<x ${index()}>] $(${ignore($tail)})*),+| {
# 				$($f).+($($($head),*,)? $([<x ${index()}>] $(, $tail)*),*)
# 			}
# 		}
# 	};
# 	($($f:ident).+ ($($head:expr_2021),*)) => {
# 		|| {
# 			$($f).+($($head),*)
# 		}
# 	};
# }
use pipeline_macro::pipe;

fn fn1(x: u16, y: u16, z: u16) -> (u16, u16, u16) {
	(x, y, z)
}

let x = 1;
let y = 2;
let z = 3;

assert_eq!(
	pipe!(x => (bind!(fn1(_, _, z))) (_, _)),
	fn1(x, x, z)
);
```

In the above example, `bind!(fn1(_, _, z))` can be evaluated like a function with two arguments.