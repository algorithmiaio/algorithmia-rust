extern crate algorithmia;
extern crate getopts;

use algorithmia::{Algorithm, Client};
use getopts::{getopts, optopt, optflag, usage, OptGroup};
use std::os;
use std::io::File;

fn print_usage(opts: &[OptGroup]) {
    print!("{}", usage("Usage: algo [options] USER/REPO", opts));
    os::set_exit_status(1);
}

fn read_file_to_string(path: Path) -> String {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {}", display, why.desc),
        Ok(file) => file,
    };

    match file.read_to_string() {
        Err(why) => panic!("could not read {}: {}", display, why.desc),
        Ok(s) => s,
    }
}

fn main() {
    let args = os::args();

    let opts = [
        getopts::optflag("h", "help", "print this help"),
        getopts::optopt("d", "data", "string to use as input data", "DATA"),
        getopts::optopt("f", "file", "file containing input data", "FILE"),
    ];

    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            print_usage(&opts);
            return;
        }
    };

    if matches.opt_present("help") || matches.free.len() == 0 {
        print_usage(&opts);
        return;
    }

    let user_repo: Vec<&str> = matches.free[0].split('/').collect();
    let data = match (matches.opt_str("data"), matches.opt_str("file")) {
        (Some(s), None) => s,
        (None, Some(f)) => read_file_to_string(Path::new(f)),
        _ => {
            println!("Must specify -f or -d");
            print_usage(&opts);
            return;
        }
    };

    let client = Client::new(env!("ALGORITHMIA_API_KEY"));
    let algorithm = Algorithm::new(user_repo[0], user_repo[1]);

    let output = match client.query_raw(algorithm, &*data) {
        Ok(result) => result,
        Err(why) => panic!("{:?}", why),
    };

    println!("{}", output);
}