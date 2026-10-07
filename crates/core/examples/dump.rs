use pendon_core::{parse, Options};
fn main() {
    for (label, src) in [
        (
            "MULTI",
            "==note[a]\n# Title\n\npara 1\n\n- item\n- item2\n==\n",
        ),
        ("DEEPNEST", "==outer\n==mid\ninner para\n==\n==\n"),
    ] {
        let events = parse(src, &Options::default());
        println!("======= {label} =======");
        for ev in &events {
            println!("{:?}", ev);
        }
    }
}
