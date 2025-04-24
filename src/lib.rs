#![cfg_attr(test, feature(macro_metavar_expr))]
#![no_std]
#![doc = include_str!("../README.md")]

/// A macro which evaluates functions from left to right, rather than from inside to outside.
///
/// Syntax: `pipe!(init => fn1 => fn2 => ...)`
///
/// Each function is either the name of a single-argument function (optionally with parentheses),
/// an expression which is parenthesizable and callable as a single-argument function (usually a lambda),
/// or a name/parenthesized expression followed by a parenthesized comma-separated list of arguments with one or more
/// arguments left as blank (`_`). All function calls and expressions to the left will be evaluated, stored in a temporary,
/// and then inserted into the current function call in place of any blanks.
#[macro_export]
macro_rules! pipe {
	($e:expr) => { $e };
	($in:expr => $($i:ident).+ $(())? $(=> $($tail:tt)+)?) => {
		pipe!($($i).+($in) $(=> $($tail)+)?)
	};
	($in:expr => $($i:ident).+ ($($($arg_head:expr_2021),*,)? $(_ $(, $arg_tail:expr_2021)*),*) $(=> $($tail:tt)+)?) => {
		{
			let pipe_temp = $in; // Eval once and cache
			pipe!($($i).+($($($arg_head),*,)? $(pipe_temp $(, $arg_tail)*),*) $(=> $($tail)+)?)
		}
	};
	($in:expr => ($e:expr) ($($($arg_head:expr_2021),*,)? $(_ $(, $arg_tail:expr_2021)*),*) $(=> $($tail:tt)+)?) => {
		{
			let pipe_temp = $in; // Eval once and cache
			pipe!($e($($($arg_head),*,)? $(pipe_temp $(, $arg_tail)*),*) $(=> $($tail)+)?)
		}
	};
	($in:expr => $e:expr $(=> $($tail:tt)+)?) => {
		pipe!($e($in) $(=> $($tail)+)?)
	};
}

#[cfg(test)]
mod tests {
	/// Tests the simple use case - piping to a function which only accepts a single argument.
	#[test]
	fn test_simple() {
		fn test(x: u16) -> f32 {
			f32::from(x + 1)
		}

		fn test2(x: f32) -> f32 {
			x + 1.
		}

		let x = 3;

		assert_eq!(pipe!(x+1 => test => test2), test2(test(x + 1)));
		assert_eq!(pipe!(x+1 => test() => test2()), test2(test(x + 1)));
	}

	/// Tests piping one argument into a function which accepts multiple arguments.
	#[test]
	fn test_underscore_fill() {
		fn test(x: u16) -> u16 {
			x
		}
		fn test2(x: u16, y: u16) -> (u16, u16) {
			(x, y)
		}

		let x = 3;
		let y = 4;

		assert_eq!(pipe!(x-1 => test => test2(_, y)), test2(test(x - 1), y));
		assert_eq!(pipe!(x-1 => test => test2(y, _)), test2(y, test(x - 1),));
		assert_eq!(
			pipe!(x-1 => test => test2(_, _)),
			test2(test(x - 1), test(x - 1))
		);
	}

	/// Make sure associated functions are callable.
	#[test]
	fn test_associated_functions() {
		fn test(x: u16) -> u16 {
			x + 1
		}

		let x = 3;

		assert_eq!(
			pipe!(x-2 => test => u16::to_be => u16::isqrt),
			u16::isqrt(u16::to_be(test(x - 2)))
		);
	}

	/// Make sure we can pipe into methods.
	#[test]
	fn test_methods() {
		fn test(x: u16) -> u16 {
			x + 1
		}

		let x = 3;
		let y = 4;
		assert_eq!(
			pipe!(x+2 => test => y.max => test),
			test(y.max(test(x + 2)))
		);

		let mut y = None;
		assert_eq!(
			pipe!(x+2 => y.map_or(_, |x| x + 2) => test),
			test(y.map_or(x + 2, |x| x + 2))
		);

		y = Some(x + 2);
		assert_eq!(
			pipe!(x+2 => y.map_or(_, |x| x + 2) => test),
			test(y.map_or(x + 2, |x| x + 2))
		);
	}

	/// Make sure we can pipe into a lambda.
	#[test]
	fn test_lambdas() {
		fn test(x: u16) -> u16 {
			x + 1
		}

		let x = 3;

		assert_eq!(
			pipe!(x => test => |x| {x - 2} => test),
			test((|x| { x - 2 })(test(x)))
		);
	}

	/// Make sure we can pipe into function-like objects returned by other macros
	#[test]
	fn test_macros() {
		use paste::paste;

		// A macro which partially evaluates a function
		macro_rules! bind {
			($($f:ident).+ ($($($head:expr_2021),*,)? $(_ $(, $tail:expr_2021)*),*)) => {
				paste! {
					|$([<x ${index()}>] $(${ignore($tail)})*),+| {
						$($f).+($($($head),*,)? $([<x ${index()}>] $(, $tail)*),*)
					}
				}
			};
			($($f:ident).+ ($($head:expr_2021),*)) => {
				|| {
					$($f).+($($head),*)
				}
			};
		}

		fn test(x: u16, y: u16, z: u16) -> (u16, u16, u16) {
			(x, y, z)
		}

		let x = 1;
		let y = 2;
		let z = 3;

		assert_eq!(pipe!(x => bind!(test(_, y, z))), test(x, y, z));
		assert_eq!(pipe!(y => bind!(test(x, _, z))), test(x, y, z));
		assert_eq!(pipe!(x => (bind!(test(_, _, z))) (_, _)), test(x, x, z));
	}
}
