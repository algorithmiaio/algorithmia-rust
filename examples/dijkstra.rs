#![feature(slice_patterns)]
extern crate algorithmia;
extern crate rustc_serialize;

use algorithmia::Service;
use algorithmia::algorithm::{Algorithm, AlgorithmOutput, Version};
use std::collections::HashMap;
use std::env;
use rustc_serialize::{json};

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

// Map of all possible points to their possible destinations (with distances)
type SrcDestMap<'a> = HashMap<&'a str, HashMap<&'a str, u32>>;

// The input format for the kenny/Dijkstra algorithm
type DijkstraInput<'a> = (SrcDestMap<'a>, &'a str, &'a str);

// The output format for the kenny/Dijkstra algorithm
type Route = Vec<String>;

struct RouteMap<'a> {
  map: SrcDestMap<'a>
}

impl<'a> RouteMap<'a> {
    pub fn get_dijkstra_route(self, start: &'a str, end: &'a str) -> AlgorithmOutput<Route> {
        let api_key = match env::var("ALGORITHMIA_API_KEY") {
            Ok(key) => key,
            Err(e) => { panic!("ERROR: unable to get ALGORITHMIA_API_KEY: {}", e); }
        };
        let service = Service::new(&*api_key);
        let dijkstra = Algorithm::new("anowell", "Dijkstra", Version::Latest);
        let dijkstra_service = service.algorithm(&dijkstra);

        println!("Making request to: {}", dijkstra.to_url());

        // Declaring type explicitly to enforce valid input types during build
        let input_data: DijkstraInput = (self.map, start, end);
        // println!("Input: {:?}", input_data);
        println!("Input:\n{}", json::as_pretty_json(&input_data));

        let output: AlgorithmOutput<Route> = match dijkstra_service.exec(&input_data) {
            Ok(out) => out,
            Err(why) => panic!("{:?}", why),
        };
        output
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (start, end) = match &*args {
        [_, ref start, ref end] => (&**start, &**end),
        _ => ("a", "c"),
    };

    let input_map = RouteMap {
        map: hashmap!(
            "a" => hashmap!("b" => 1),
            "b" => hashmap!("a" => 2, "c" => 2),
            "c" => hashmap!("b" => 2, "d" => 1),
            "d" => hashmap!("a" => 1, "c" => 3)
        )
    };

    let output = input_map.get_dijkstra_route(start, end);
    println!("Shortest route: {:?}\nCompleted in {} seconds.", output.result, output.duration);
}
