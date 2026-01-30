#[macro_export]
macro_rules! combinator {
	(S) => { $crate::Combinator::S };
	(K) => { $crate::Combinator::K };
	(Y) => { $crate::Combinator::Y };
	(T) => { $crate::Combinator::T };
	(+) => { $crate::Combinator::Add };
	(=) => { $crate::Combinator::Eq };
	(?) => { $crate::Combinator::Read };
	(!) => { $crate::Combinator::Show };
  ($x:literal) => {
    $crate::Combinator::N($x as i32)
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
