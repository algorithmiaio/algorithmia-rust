#![feature(core)]

extern crate algorithmia;
extern crate getopts;

use algorithmia::{Service};
use getopts::Options;
use std::ascii::AsciiExt;
use std::env;
use std::fs::File;
// use std::thread;

fn print_usage(opts: &Options) {
    let brief = vec![
        "Usage: algodata USER/COLLECTION [CMD [CMD_ARGS...]]",
        "Supported CMDs",
        "  CREATE",
        "  UPLOAD FILE..."
    ];
    println!("{}", opts.usage(&*brief.connect("\n")));
    env::set_exit_status(1);
}

struct AlgoData<'a> {
    service: Service<'a>,
}

impl<'a> AlgoData<'a> {
    fn new(api_key: &'a str) -> AlgoData<'a> {
        AlgoData { service: Service::new(api_key) }
    }

    fn show_collection(self, username: &str, collection_name: &str) {
        let mut my_bucket = self.service.collection(username, collection_name);
        match my_bucket.show() {
            Ok(output) => println!("{:?}", output),
            Err(why) => println!("ERROR: {:?}", why),
        };
    }

    fn create_collection(self, username: &str, collection_name: &str) {
        let mut my_bucket = self.service.collection(username, collection_name);
        match my_bucket.create() {
            Ok(output) => println!("{:?}", output),
            Err(why) => println!("ERROR: {:?}", why),
        };
    }

    fn upload_files(self, username: &str, collection_name: &str, file_paths: &[String]) {
        let mut my_bucket = self.service.collection(username, collection_name);
        for file_path in file_paths {
            println!("Uploading {}", file_path);
            match File::open(file_path) {
                Ok(mut file) => {
                    let ref mut bucket = my_bucket;
                    bucket.upload_file(&mut file);
                },
                Err(e) => {
                    println!("Failed to open {}: {}", file_path, e)
                },
            };
        }
        println!("Finished uploading {} file(s)", file_paths.len())
    }
}


fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help");

    let args = match opts.parse(env::args()) {
        Ok(m) => m,
        Err(f) => {
            println!("Failed to parse args: {}", f);
            print_usage(&opts);
            return;
        }
    };

    if args.opt_present("help") || args.free.len() == 0 {
        print_usage(&opts);
        return;
    }

    let api_key = match env::var("ALGORITHMIA_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            println!("Must set ALGORITHMIA_API_KEY");
            print_usage(&opts);
            return;
        }
    };



    let data = AlgoData::new(&*api_key);
    let mut args_iter = args.free.into_iter();

    // Throwout program arg
    args_iter.next();

    let first_arg = args_iter.next();
    let user_collection: Vec<&str> = match first_arg {
        Some(ref arg) => arg.split('/').collect(),
        None => {
            println!("Did not specity USERNAME/COLLECTION");
            print_usage(&opts);
            return;
        }
    };
    let cmd = match args_iter.next() {
        Some(ref arg) => arg.to_ascii_lowercase(),
        None => "show".to_string(),
    };

    match user_collection.as_slice() {
        [user, collection] => {
            match cmd.as_slice() {
                "show" => data.show_collection(user, collection),
                "create" => data.create_collection(user, collection),
                "upload" => {
                    let files: Vec<String> = args_iter.collect();
                    data.upload_files(user, collection, files.as_slice());
                },
                invalid => {
                    println!("Not a valid command: {}", invalid);
                    print_usage(&opts);
                    return;
                }
            }
        },
        invalid => {
            println!("Invalid data repository: {:?}", invalid );
            print_usage(&opts);
            return;
        }
    }
}