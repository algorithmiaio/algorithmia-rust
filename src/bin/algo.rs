#![feature(collections)]
#![feature(fs)]
#![feature(io)]
#![feature(os)]
#![feature(old_path)]

extern crate algorithmia;
extern crate getopts;

use algorithmia::Service;
use getopts::Options;
use std::env;
use std::io::Read;
use std::os;
use std::fs::File;

fn print_usage(opts: &Options) {
    print!("{}", opts.usage("Usage: algo [options] USER/REPO"));
    env::set_exit_status(1);
}

fn read_file_to_string(path: Path) -> String {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {:?}", display, why),
        Ok(file) => file,
    };

    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Err(why) => panic!("could not read {}: {:?}", display, why),
        Ok(s) => s,
    }
    data
}

fn main() {
    let args = os::args();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("d", "data", "string to use as input data", "DATA");
    opts.optopt("f", "file", "file containing input data", "FILE");

    let matches = match opts.parse(args.tail()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            print_usage(&opts);
            return;
        }
    };

    let api_key = match env::var("ALGORITHMIA_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            println!("Must set ALGORITHMIA_API_KEY");
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

    let service = Service::new(&*api_key);
    let mut algorithm = service.algorithm(user_repo[0], user_repo[1]);

    let output = match algorithm.query_raw(&*data) {
        Ok(result) => result,
        Err(why) => panic!("{:?}", why),
    };

    println!("{}", output);
}