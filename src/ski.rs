use crate::combinator;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Combinator {
    S,
    K,
    Y,

    T, // Tuple: T n a_1 .. a_n f ~> f a_1 .. a_n
    Add,
    Eq,

    // IO
    Read, // Read (\b -> f) ~> f n [where n is a byte read from stdin]
    Show, // Show b c       ~> c   [where n is a byte printed to stdout]

    N(i32), // Number

    App(Vec<Combinator>), // Combinators are stored in reverse
    Named(&'static str, Box<Combinator>),
}

impl Combinator {
    #[rustfmt::skip]
    pub const BASIS: &[(&'static str, Combinator, usize)] = &[
        ("S", Self::S, 3),
        ("K", Self::K, 2),
        ("Y", Self::Y, 1),
        ("+", Self::Add, 2),
        ("=", Self::Eq, 2),
        ("T", Self::T, 1),
        ("$", Self::Read, 1),
        ("!", Self::Show, 2),
    ];

    pub fn normal_form(&self, limit: Option<usize>) -> Option<Self> {
        let mut copy = self.clone();

        if copy.normalize(limit) {
            Some(copy)
        } else {
            None
        }
    }

    // Put everything in terms of S & K
    pub fn simplify(&mut self) {
        use Combinator::*;
        match self {
            Named(_, def_box) => {
                let def = std::mem::replace(def_box.as_mut(), S);
                *self = def;
                self.simplify()
            }
            App(terms) => match &terms[..] {
                [_] | [.., App(_)] => {
                    self.reduce();
                    self.simplify()
                }
                _ => {
                    for term in terms {
                        term.simplify()
                    }
                }
            },
            _ => (),
        }
    }

    pub fn normalize(&mut self, mut limit: Option<usize>) -> bool {
        self.normalize_with(&mut limit)
    }

    fn normalize_with(&mut self, limit: &mut Option<usize>) -> bool {
        if *limit == Some(0) {
            return false;
        }

        if let Some(limit) = limit {
            *limit -= 1;
        }

        use Combinator::*;
        match self {
            Named(_, def) => def.normalize_with(limit),

            App(terms) => match &terms[..] {
                [] => unreachable!(),
                [_] => {
                    *self = terms.pop().unwrap();
                    self.normalize_with(limit)
                }
                [.., App(_)] => {
                    let App(head) = terms.pop().unwrap() else {
                        unreachable!()
                    };
                    terms.extend(head);
                    self.normalize_with(limit)
                }

                [.., Named(_, _)] => {
                    let named = terms.last_mut().unwrap();
                    if !named.normalize_with(limit) {
                        return false;
                    }

                    self.reduce();
                    self.normalize_with(limit)
                }
                [.., _f, Read] => {
                    let _ = terms.pop().unwrap();
                    let f = terms.pop().unwrap();

                    // Read one byte from stdin
                    let b = {
                        use std::io::Read;
                        let mut buf = [0];
                        std::io::stdin().read_exact(&mut buf).unwrap();
                        buf[0] as i32
                    };

                    terms.push(N(b));
                    terms.push(f);

                    self.normalize_with(limit)
                }
                [.., _c, _n, Show] => {
                    let _ = terms.pop().unwrap();
                    let mut b = terms.pop().unwrap();

                    if !b.normalize_with(limit) {
                        return false;
                    }
                    let N(b) = b else { unreachable!() };

                    print!("{}", b as u8 as char);
                    self.normalize_with(limit)
                }
                [.., _f, Y] => {
                    let _ = terms.pop().unwrap();
                    let f = terms.pop().unwrap();

                    terms.push(App(vec![f.clone(), Y]));
                    terms.push(f);
                    self.normalize_with(limit)
                }
                [ps @ .., _, T] => {
                    let nargs = ps.len();
                    let len = terms.len();
                    let n = &mut terms[len - 2];
                    if !n.normalize_with(limit) {
                        return false;
                    }
                    let N(n) = n else { unreachable!() };
                    let n = *n;
                    if n < 0 {
                        return false;
                    }

                    if nargs < (n + 1) as usize {
                        return terms.iter_mut().all(|n| n.normalize_with(limit));
                    }

                    let _t = terms.pop();
                    let _n = terms.pop();

                    let last = terms.len() - 1;
                    let f = terms.remove(last - n as usize);
                    terms.push(f);

                    self.normalize_with(limit)
                }
                [.., N(n)] if *n < 0 => false,
                [.., _, _, N(0)] => {
                    let _n = terms.pop().unwrap();
                    let _f = terms.pop().unwrap();

                    self.normalize_with(limit)
                }
                [.., _, _, N(n)] => {
                    let rec = N(*n - 1);

                    let _n = terms.pop().unwrap();
                    let f = terms.pop().unwrap();
                    let x = terms.pop().unwrap();

                    #[rustfmt::skip]
                    terms.push(App(vec![
                        App(vec![x, f.clone(), rec]),
                        f
                    ]));
                    self.normalize_with(limit)
                }
                [.., _, _, Eq] => {
                    let at = terms.len() - 3;
                    let mut args = terms.split_off(at);
                    let _ = args.pop();
                    let mut p = args.pop().unwrap();
                    let mut q = args.pop().unwrap();

                    p.normalize_with(limit);
                    q.normalize_with(limit);

                    use Combinator::N;
                    let (N(p), N(q)) = (p, q) else { unreachable!() };

                    // T x y ~> x
                    // F x y ~> y
                    if p == q {
                        terms.push(K);
                    } else {
                        terms.push(combinator!(K (S K K)));
                    }
                    self.normalize_with(limit)
                }
                [.., _, _, Add] => {
                    let at = terms.len() - 3;
                    let mut args = terms.split_off(at);
                    let _ = args.pop();
                    let mut p = args.pop().unwrap();
                    let mut q = args.pop().unwrap();

                    if !p.normalize_with(limit) || !q.normalize_with(limit) {
                        return false;
                    }

                    use Combinator::N;
                    let (N(p), N(q)) = (p, q) else { unreachable!() };

                    terms.push(N(p + q));
                    self.normalize_with(limit)
                }
                [.., _x, _g, _f, S] => {
                    let _s = terms.pop().unwrap();
                    let mut f = terms.pop().unwrap();
                    let mut g = terms.pop().unwrap();
                    let x = terms.pop().unwrap();

                    g.apply(x.clone());
                    terms.push(g);
                    f.apply(x);
                    terms.push(f);
                    self.normalize_with(limit)
                }

                [.., _y, _x, K] => {
                    let _k = terms.pop().unwrap();
                    let x = terms.pop().unwrap();
                    let _y = terms.pop().unwrap();

                    terms.push(x);

                    self.normalize_with(limit)
                }

                _ => terms
                    .iter_mut()
                    .rev()
                    .all(|term| term.normalize_with(limit)),
            },
            _ => true,
        }
    }

    pub fn reduce(&mut self) -> bool {
        match self {
            Self::Named(_, inner) => {
                let old_inner = std::mem::replace(inner.as_mut(), Self::S);
                *self = old_inner;
                true
            }
            Self::App(terms) => match &mut terms[..] {
                [_] => {
                    *self = terms.pop().unwrap();
                    self.reduce()
                }
                [.., _y, _x, Self::K] => {
                    terms.pop();
                    let x = terms.pop().unwrap();
                    terms.pop();

                    //For efficiency reasons, to avoid reallocation
                    //and an extra reduction step
                    if let Self::App(v) = x {
                        terms.extend_from_slice(&v);
                    } else {
                        terms.push(x);
                    }
                    true
                }
                [.., _x, _g, _f, Self::S] => {
                    terms.pop();
                    let mut f = terms.pop().unwrap();
                    let mut g = terms.pop().unwrap();
                    let x = terms.pop().unwrap();

                    g.apply(x.clone());
                    terms.push(g);

                    //For efficiency reasons, to avoid reallocation
                    //and an extra reduction step
                    f.apply(x);
                    if let Self::App(v) = f {
                        terms.extend_from_slice(&v);
                    } else {
                        terms.push(f);
                    }

                    true
                }
                [.., Self::App(_)] => {
                    let Some(Self::App(inner)) = terms.pop() else {
                        unreachable!()
                    };

                    terms.extend_from_slice(&inner[..]);
                    self.reduce()
                }
                [.., Self::Named(_, _)] => {
                    let Some(Self::Named(_, top)) = terms.pop() else {
                        unreachable!()
                    };

                    if let Self::App(v) = *top {
                        terms.extend_from_slice(&v);
                    } else {
                        terms.push(*top);
                    }

                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    //Apply without additional copy
    pub fn apply(&mut self, x: Self) {
        if let Self::App(args) = self {
            //Push x to front of args, without copy or reallocation
            args.reserve(1);
            let mut tmp = x;
            for arg in args.iter_mut() {
                std::mem::swap(&mut tmp, arg);
            }
            args.push(tmp);
        } else {
            *self = Self::App(vec![x, self.clone()]);
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::App(args) => args.iter().map(Combinator::size).sum(),
            _ => 1,
        }
    }
}

impl std::fmt::Display for Combinator {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Combinator::*;
        match self {
            Named(name, _) => write!(fmt, "{}", name),
            App(combs) => {
                for comb in combs.iter().rev() {
                    if comb.size() != 1 || matches!(comb, N(_)) {
                        write!(fmt, "({})", comb)?;
                    } else {
                        write!(fmt, "{}", comb)?;
                    }
                }
                Ok(())
            }
            N(n) => write!(fmt, "{}", n),
            c => match Combinator::BASIS.iter().find(|s| &s.1 == c) {
                Some(c) => write!(fmt, "{}", c.0),
                _ => unreachable!(),
            },
        }
    }
}
