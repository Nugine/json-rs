use std::io::Read;

fn main() -> std::io::Result<()> {
    let mut s = String::new();
    std::io::stdin().lock().read_to_string(&mut s)?;

    match json_rs::parse(&s) {
        Ok(val) => {
            println!("{:?}", val);
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    };
    Ok(())
}
