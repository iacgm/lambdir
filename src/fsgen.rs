use crate::*;
use std::fs::*;
use std::io::Error;
use std::path::*;

pub fn gen_fs<P: AsRef<Path>>(name: P, combinator: &Combinator) -> Result<(), Error> {
    let path = name.as_ref();
    let _ = remove_dir_all(path);
    create_dir(path)?;

    use Combinator::*;
    match combinator {
        S => create_dir(path.join("S")),
        K => create_dir(path.join("K")),
        Var(v) => create_dir(path.join(format!("{}", v))),
        Named(_, combinator) => gen_fs(path, combinator),
        App(combinators) => {
            for (i, comb) in combinators.iter().enumerate() {
                gen_fs(path.join(format!("{}", i)), comb)?;
            }
            Ok(())
        }
    }
}

pub fn read_fs<P: AsRef<Path>>(name: P) -> Result<Combinator, Error> {
    let path = name.as_ref();
    let entries = ls_dir(path);

    use Combinator::*;
    match &entries[..] {
        [] => Err(Error::other("Empty Combinator.")),
        [path] => {
            let name = get_name(path);
            match Combinator::BASIS.iter().find(|s| s.0 == name) {
                Some(c) => Ok(c.1.clone()),
                _ => Err(Error::other(format!("Unrecognized symbol: `{}`", name))),
            }
        }
        [ps @ ..] => {
            let combs = ps
                .iter()
                .map(|p| {
                    let name = get_name(p);
                    let num = name.parse::<u32>();
                    assert!(num.is_ok(), "Malformed position ID: `{:?}`", num);

                    read_fs(path.join(name))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;

            Ok(App(combs))
        }
    }
}

pub fn ls_dir<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    let path = path.as_ref();
    let mut entries: Vec<PathBuf> = read_dir(path)
        .unwrap()
        .into_iter()
        .map(Result::unwrap) // Who even cares?
        .map(|c| c.path())
        .collect::<Vec<_>>();
    // We only care about sorting numerical directories
    entries.sort_by_key(|p| get_name(p).parse::<u32>().unwrap_or(0));

    entries
}

pub fn get_name<'a>(path: &'a PathBuf) -> &'a str {
    path.components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
}
