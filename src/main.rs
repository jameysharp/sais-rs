use sais::sais_utf8;

fn main() {
    for s in std::env::args().skip(1) {
        println!("suffixes of {s}:");
        let sa = sais_utf8(&s);
        for (idx, suffix) in sa.into_iter().enumerate() {
            println!("{idx}: {:?}", &s[suffix..]);
        }
    }
}
