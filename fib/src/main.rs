use std::env;
use std::collections::HashMap;

fn calc_fib_number(idx: i64) -> i128{
    // naive implementation - very slow. Left in to demonstrate, but will produce a compiler warning
    // because it is dead code
    if idx == 0{
        return 0
    }else if idx == 1 {
        return 1
    }
    return calc_fib_number(idx - 1) + calc_fib_number(idx - 2)
}

fn calc_fib_number_cache(idx: i64, mut cache: HashMap<i64, i128>) -> (i128, HashMap<i64, i128>){
    let lookup = cache.get(&idx);
    match lookup{
        Some(val) => {
            // values in the cache are just returning
            (*val, cache)
        }
        None => {
            // have to reassign cache in between calls to avoid borrow checker issues. This way, each
            // value is only moved once
            let first_result = calc_fib_number_cache(idx - 1, cache);
            let first = first_result.0;
            // cache was moved in call to a function - has to be re-assigned so we can use in second call
            cache = first_result.1;

            let second_result = calc_fib_number_cache(idx - 2, cache);
            let second = second_result.0;
            // call to second function moved again, reassign the result here
            cache = second_result.1;

            let val = first + second;
            cache.insert(idx, val);
            (val, cache)
        }
    }
}

fn main() {
    let args:Vec<String> = env::args().collect();
    let mut cache:HashMap<i64, i128> = HashMap::from([(0, 0), (1,1)]);
    for i in 1..args.len(){
        let arg = &args[i];
        let parse_result = arg.parse::<i64>();
        let int_arg: i64;
        match parse_result {
            Ok(parse_result) => {
                int_arg = parse_result;
                if int_arg < 0{
                    eprintln!("Error: arg {} is a negative value. Only positive values allowed.", arg);
                    continue
                }
                // calc_fib_number_cache borrows the value of cache, so we need to "give it back"
                // in the return
                let result = calc_fib_number_cache(int_arg, cache);
                let fib_num = result.0;
                cache = result.1;
                println!("{}: {}", int_arg, fib_num)
            }
            Err(parse_result) => {
                // this case represents an error - send to stderr instead of stdout
                eprintln!("Error: {:?} unwrapping arg {}, will not compute", parse_result, arg)
            }
        }
    }
}
