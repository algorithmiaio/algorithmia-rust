#![feature(collections)]
#![feature(fs)]
#![feature(io)]
#![feature(os)]
#![feature(old_path)]

extern crate algorithmia;
extern crate getopts;

use algorithmia::{Service};
use getopts::Options;
use std::ascii::AsciiExt;
use std::io::Read;
use std::os;
use std::fs::File;
use std::thread;

fn print_usage(opts: &Options) {
    print!("{}", opts.usage("Usage: datatool USER/COLLECTION [CMD]"));
    os::set_exit_status(1);
}

struct AlgoData<'a> {
    service: Service<'a>,
}

impl<'a> AlgoData<'a> {
    fn new() -> AlgoData<'a> {
        AlgoData { service: Service::new(env!("ALGORITHMIA_API_KEY")) }
    }

    fn create_collection(mut self, username: &str, collection_name: &str) {
        let mut my_bucket = self.service.collection(username, collection_name);
        match my_bucket.create() {
            Ok(output) => println!("{:?}", output),
            Err(why) => println!("ERROR: {:?}", why),
        };
    }

    fn upload_files(mut self, username: &str, collection_name: &str, file_paths: &[String]) {
        let mut my_bucket = self.service.collection(username, collection_name);
        for file_path in file_paths {
            println!("Uploading {}", file_path);
            let mut file = File::open(file_path).unwrap();
            let ref mut bucket = my_bucket;
            bucket.upload_file(&mut file);
        }
        println!("Finished uploading {} file(s)", file_paths.len())
    }
}


fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");

    let args = match opts.parse(os::args().tail()) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            print_usage(&opts);
            return;
        }
    };

    if args.opt_present("help") || args.free.len() == 0 {
        print_usage(&opts);
        return;
    }

    let data = AlgoData::new();
    let foo: Vec<&str> = args.free[0].split('/').collect();
    let cmd = args.free[1].to_ascii_lowercase();

    match foo.as_slice() {
        [user, collection] => {
            match cmd.as_slice() {
                "create" => data.create_collection(user, collection),
                "upload" => data.upload_files(user, collection, args.free[2..].as_slice()),
                _ => { print_usage(&opts); return; }
            }
        },
        _ => { print_usage(&opts); return; }
    }
}