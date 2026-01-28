use std::fs::*;
use std::io::Error;
use std::path::*;

use crate::*;

pub fn exec_fs<P: AsRef<Path>>(path: P) {
    while reduce_fs(&path) {}
}

// No attempt made for error handling
pub fn reduce_fs<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let children = ls_dir(path);

    match &children[..] {
        [] => unreachable!(),
        [p] => match get_name(p) {
            "0" => {
                safe_rename(p, "tmp");
                remove_dir_all(path).unwrap();
                safe_rename("tmp", path);
                true
            }
            _ => false,
        },
        [ps @ ..] => {
            let count = ps.len() - 1;
            let count_str = format!("{}", count);
            let head_path = &path.join(count_str);
            let head = ls_dir(head_path);

            let nth = |n: usize| path.join(format!("{}", count - n));

            // 'Unparenthesize' (i.e., flatten) head
            match &head[..] {
                [] => unreachable!(),
                [c] => {
                    use Combinator::*;

                    let name = get_name(c);
                    let (comb, n) = match Combinator::BASIS.iter().find(|s| s.0 == name) {
                        Some(("T", _, _)) => {
                            if count == 0 {
                                return false;
                            }

                            let arg = &nth(1);
                            if reduce_fs(arg) {
                                return true;
                            }

                            let narg = &ls_dir(arg)[0];
                            let n = get_name(narg)[1..].parse::<i32>().unwrap();
                            if n < 0 {
                                return false;
                            }

                            // T n =~ \a_1 .. a_n f -> f a_1 .. a_n
                            (T, n as usize + 2)
                        }
                        Some((_, comb, n)) => (comb.clone(), *n),
                        None if &name[..1] == "N" => {
                            let n = name[1..].parse::<i32>().unwrap();
                            if n < 0 {
                                return false;
                            }

                            (N(n), 2) // n acts as \f x -> f^n x
                        }
                        None => {
                            // Not in head normal form
                            safe_rename(c, "tmp");
                            remove_dir_all(head_path).unwrap();
                            safe_rename("tmp", head_path);
                            return true;
                        }
                    };

                    if count < n {
                        return ps.iter().any(reduce_fs);
                    }

                    let out_path = nth(n);

                    match comb {
                        S => {
                            let tmp = path.join("tmp");
                            create_dir(&tmp).unwrap();
                            let x = nth(1);
                            let y = nth(2);
                            let z = nth(3);

                            copy_dir(&x, tmp.join("2")).unwrap();
                            copy_dir(&z, tmp.join("1")).unwrap();

                            let a = tmp.join("0");
                            create_dir(&a).unwrap();
                            copy_dir(&y, a.join("1")).unwrap();
                            copy_dir(&z, a.join("0")).unwrap();

                            remove_dir_all(head_path).unwrap();
                            remove_dir_all(x).unwrap();
                            remove_dir_all(y).unwrap();
                            remove_dir_all(z).unwrap();

                            safe_rename(&tmp, out_path);
                        }
                        K => {
                            remove_dir_all(head_path).unwrap();
                            remove_dir_all(nth(2)).unwrap();

                            let x = nth(1);
                            safe_rename(x, out_path);
                        }
                        Eq => {
                            let x = &nth(1);
                            if reduce_fs(x) {
                                return true;
                            }

                            let y = &nth(2);
                            if reduce_fs(y) {
                                return true;
                            }

                            remove_dir_all(head_path).unwrap();
                            let n = |p: &PathBuf| {
                                let children = ls_dir(p);
                                assert_eq!(children.len(), 1);

                                let name = get_name(&children[0]);
                                assert_eq!(&name[..1], "N");
                                remove_dir_all(p).unwrap();

                                name[1..].parse::<i32>().unwrap();
                            };

                            let p = n(x);
                            let q = n(y);

                            let _ = remove_dir_all(&out_path);
                            create_dir(&out_path).unwrap();
                            if p == q {
                                create_dir(out_path.join("K")).unwrap();
                            } else {
                                create_dir_all(out_path.join("1/K")).unwrap();
                                create_dir_all(out_path.join("0/2/S")).unwrap();
                                create_dir_all(out_path.join("0/1/K")).unwrap();
                                create_dir_all(out_path.join("0/0/K")).unwrap();
                            }
                        }
                        Add => {
                            let x = &nth(1);
                            if reduce_fs(x) {
                                return true;
                            }

                            let y = &nth(2);
                            if reduce_fs(y) {
                                return true;
                            }

                            remove_dir_all(head_path).unwrap();
                            let mut s = 0;
                            for p in &[x, y] {
                                let children = ls_dir(p);
                                assert_eq!(children.len(), 1);

                                let name = get_name(&children[0]);
                                assert_eq!(&name[..1], "N");

                                s += name[1..].parse::<i32>().unwrap();
                                remove_dir_all(p).unwrap();
                            }
                            let _ = remove_dir_all(&out_path);
                            create_dir(&out_path).unwrap();
                            create_dir(out_path.join(format!("N{}", s))).unwrap();
                        }
                        N(0) => {
                            remove_dir_all(head_path).unwrap();
                            remove_dir_all(nth(1)).unwrap();
                        }
                        N(n) => {
                            let rec = format!("N{}", n - 1);
                            let f = nth(1);
                            let x = nth(2);

                            let tmp = &path.join("tmp");
                            create_dir(tmp).unwrap();
                            create_dir_all(tmp.join("2").join(rec)).unwrap();
                            safe_rename(f, tmp.join("1"));
                            safe_rename(x, tmp.join("0"));

                            remove_dir_all(head_path).unwrap();
                            copy_dir(tmp.join("1"), out_path.join("1")).unwrap();
                            safe_rename(tmp, out_path.join("0"));
                        }
                        T => {
                            let cont = &nth(n);
                            let tmp = &path.join("tmp");
                            safe_rename(cont, tmp);
                            for i in (3..=n).rev() {
                                safe_rename(nth(i - 1), nth(i));
                            }
                            safe_rename(tmp, nth(2));
                            remove_dir_all(nth(1)).unwrap();
                            remove_dir_all(head_path).unwrap();
                        }
                        _ => unreachable!(),
                    }
                    true
                }
                [cs @ ..] => {
                    let tmpd = &path.join("tmp");
                    create_dir(tmpd).unwrap();
                    for (i, c) in cs.iter().enumerate() {
                        let tmpc = tmpd.join(format!("{}", count + i));
                        safe_rename(c, tmpc);
                    }
                    for (i, _) in cs.iter().enumerate() {
                        let tmpc = tmpd.join(format!("{}", count + i));
                        let newc = path.join(format!("{}", count + i));
                        safe_rename(tmpc, newc);
                    }
                    remove_dir_all(tmpd).unwrap();
                    true
                }
            }
        }
    }
}

fn safe_rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) {
    while rename(&from, &to).is_err() {}
}

fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), Error> {
    let from = from.as_ref();
    let to = to.as_ref();
    create_dir_all(to)?;
    for c in &ls_dir(from) {
        copy_dir(c, to.join(get_name(c)))?;
    }

    Ok(())
}
