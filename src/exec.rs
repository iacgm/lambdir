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
                rename(p, "tmp").unwrap();
                remove_dir_all(path).unwrap();
                rename("tmp", path).unwrap();
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
                    let name = get_name(c);
                    let Some((_, comb, n)) = Combinator::BASIS.iter().find(|s| s.0 == name) else {
                        // Not in head normal form
                        rename(c, "tmp").unwrap();
                        remove_dir_all(head_path).unwrap();
                        rename("tmp", head_path).unwrap();
                        return true;
                    };
                    if count < *n {
                        return false;
                    }

                    let out_path = nth(*n);

                    use Combinator::*;
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

                            rename(&tmp, out_path).unwrap();
                        }
                        K => {
                            let x = nth(1);

                            remove_dir_all(head_path).unwrap();
                            remove_dir_all(nth(2)).unwrap();
                            rename(x, out_path).unwrap();
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
                        rename(c, tmpc).unwrap();
                    }
                    for (i, _) in cs.iter().enumerate() {
                        let tmpc = tmpd.join(format!("{}", count + i));
                        let newc = path.join(format!("{}", count + i));
                        rename(tmpc, newc).unwrap();
                    }
                    remove_dir_all(tmpd).unwrap();
                    true
                }
            }
        }
    }
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
