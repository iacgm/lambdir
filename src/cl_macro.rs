#[macro_export]
macro_rules! combinator {
	(S) => { $crate::Combinator::S };
	(K) => { $crate::Combinator::K };
	(T) => { $crate::Combinator::T };
	(+) => { $crate::Combinator::Add };
	(=) => { $crate::Combinator::Eq };
  ($x:literal) => {
    $crate::Combinator::N($x)
  };
	($x:ident) => {
		$crate::Combinator::Named(&stringify!($x), Box::new($x.clone()))
	};
	(($($r:tt)*)) => {
		combinator!($($r)*)
	};
	($($r:tt)*) => {{
		let mut combs = vec![$(combinator!($r)),*];
		combs.reverse();
		$crate::Combinator::App(combs)
	}};
}
