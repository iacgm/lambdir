#[macro_export]
macro_rules! combinator {
	(S) => {
		$crate::Combinator::S
	};
	(K) => {
		$crate::Combinator::K
	};
	($x:literal) => {
		$crate::Combinator::Var($x)
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

