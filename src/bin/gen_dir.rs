use lambdir::*;

fn main() -> Result<(), std::io::Error> {
    let ski = combinator!(S + (K 2) 2);

    gen_fs("ski_dir", &ski)
}
