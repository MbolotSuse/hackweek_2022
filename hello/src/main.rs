use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut names = String::from("");
    for i in 1..args.len(){
        names += &*args[i];
        if i == args.len() - 2{
            names += ", and "
        }else if i != args.len() - 1 {
            names += ", "
        }
    }
    println!("Hello {}!", names);
}
