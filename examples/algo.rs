extern crate algorithmia;

use algorithmia::{Algorithm, Client};
use std::os;

fn main() {
    let args = os::args();
    match args.len() {
        3 => (),
        _ => {
            println!("Usage: ALGORITHMIA_API_KEY=1234567890 algo USER/REPO INPUT");
            return;
        }
    };

    let user_repo = &*args[1].split_str("/").collect::<Vec<&str>>();
    let data = args[2].as_slice();

    let client = Client::new(env!("ALGORITHMIA_API_KEY"));
    let algorithm = Algorithm::new(user_repo[0], user_repo[1]);

    let output = match client.query_raw(algorithm, data) {
        Ok(result) => result,
        Err(why) => panic!("{}", why),
    };

    println!("{}", output);
}