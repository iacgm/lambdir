#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Combinator {
    S,
    K,
    // Combinators are stored in reverse
    App(Vec<Combinator>),
    // Basically for testing/debugging
    Var(char),
    // QoL
    Named(&'static str, Box<Combinator>),
}

impl Combinator {
    #[rustfmt::skip]
    pub const BASIS: &[(&'static str, Combinator, usize)] = &[
        ("S", Self::S, 3),
        ("K", Self::K, 2),
    ];

    pub fn normal_form(&self, limit: usize) -> Option<Self> {
        let mut copy = self.clone();

        if copy.normalize(limit) {
            Some(copy)
        } else {
            None
        }
    }

    // Put everything in terms of S & K
    pub fn sk_ify(&mut self) {
        match self {
            Self::S | Self::K | Self::Var(_) => (),
            Self::Named(_, def_box) => {
                let def = std::mem::replace(def_box.as_mut(), Self::S);
                *self = def;
                self.sk_ify()
            }
            Self::App(terms) => match &terms[..] {
                [_] | [.., Self::App(_)] => {
                    self.reduce();
                    self.sk_ify()
                }
                _ => {
                    for term in terms {
                        term.sk_ify()
                    }
                }
            },
        }
    }

    pub fn normalize(&mut self, mut limit: usize) -> bool {
        assert!(limit > 0);
        self.normalize_with(&mut limit)
    }

    fn normalize_with(&mut self, limit: &mut usize) -> bool {
        if *limit == 0 {
            return false;
        }

        *limit -= 1;

        match self {
            Self::S | Self::K | Self::Var(_) => true,

            Self::Named(_, def) => def.normalize_with(limit),

            Self::App(terms) => match &terms[..] {
                [] => unreachable!(),
                [_] => {
                    *self = terms.pop().unwrap();
                    true
                }
                [.., Self::App(_)] => {
                    let Self::App(head) = terms.pop().unwrap() else {
                        unreachable!()
                    };
                    terms.extend(head);
                    self.normalize_with(limit)
                }

                [.., Self::Named(_, _)] => {
                    let named = terms.last_mut().unwrap();
                    named.normalize_with(limit);

                    self.reduce();
                    self.normalize_with(limit)
                }

                [_, _, _, Self::S] | [_, _, Self::K] => {
                    self.reduce();
                    self.normalize_with(limit)
                }

                [.., _x, _g, _f, Self::S] => {
                    let _s = terms.pop().unwrap();
                    let f = terms.pop().unwrap();
                    let g = terms.pop().unwrap();
                    let x = terms.pop().unwrap();

                    terms.push(Self::App(vec![x, g, f, Self::S]));

                    let redex = terms.last_mut().unwrap();

                    if redex.normalize_with(limit) {
                        self.normalize_with(limit)
                    } else {
                        false
                    }
                }

                [.., _y, _x, Self::K] => {
                    let _k = terms.pop().unwrap();
                    let x = terms.pop().unwrap();
                    let y = terms.pop().unwrap();

                    terms.push(Self::App(vec![y, x, Self::K]));

                    let redex = terms.last_mut().unwrap();

                    if redex.normalize_with(limit) {
                        self.normalize_with(limit)
                    } else {
                        false
                    }
                }

                _ => terms
                    .iter_mut()
                    .rev()
                    .all(|term| term.normalize_with(limit)),
            },
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
        match self {
            Self::S => write!(fmt, "S"),
            Self::K => write!(fmt, "K"),
            Self::Var(var) => write!(fmt, "{}", var),
            Self::Named(name, _) => write!(fmt, "{}", name),
            Self::App(combs) => {
                for comb in combs.iter().rev() {
                    if comb.size() == 1 {
                        write!(fmt, "{}", comb)?;
                    } else {
                        write!(fmt, "({})", comb)?;
                    }
                }
                Ok(())
            }
        }
    }
}
