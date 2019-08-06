use std::io::BufRead;

fn main() {
    let mut s = String::new();

    loop {
        let size = std::io::stdin().lock().read_line(&mut s).unwrap();
        if size == 0 {
            break;
        }
        println!("{:?}", &s);
        let res = json_rs::parse(&s);
        println!("{:?}", res);
        s.clear();
    }
}
