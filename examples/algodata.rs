#![feature(collections)]
#![feature(fs)]
#![feature(io)]
#![feature(os)]
#![feature(old_path)]

extern crate algorithmia;
extern crate getopts;

use algorithmia::Service;
use getopts::Options;
use std::io::Read;
use std::os;
use std::fs::File;

fn print_usage(opts: &Options) {
    print!("{}", opts.usage("Usage: datatool USER/COLLECTION [CMD]"));
    os::set_exit_status(1);
}

fn main() {
    let args = os::args();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");

    let matches = match opts.parse(args.tail()) {
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

    let user_collection: Vec<&str> = matches.free[0].split('/').collect();

    let service = Service::new(env!("ALGORITHMIA_API_KEY"));
    let mut my_bucket = service.collection(user_collection[0], user_collection[1]);

    match my_bucket.create() {
        Ok(output) => println!("{:?}", output),
        Err(why) => println!("ERROR: {:?}", why),
    };
}