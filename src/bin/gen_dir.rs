use lambdir::*;

fn main() -> Result<(), std::io::Error> {
    let ski = combinator!(S (K K) (K S K));

    gen_fs("ski_dir", &ski)
}
