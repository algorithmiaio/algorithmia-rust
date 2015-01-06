extern crate algorithmia;
extern crate "rustc-serialize" as rustc_serialize;

use algorithmia::{Algorithm, Client, Output};
use std::collections::HashMap;
use rustc_serialize::{json};

type VertexMap<'a> = HashMap<&'a str, Vec<int>>;
type EdgeMap<'a> = HashMap<&'a str, Vec<&'a str>>;

// The input format for the kenny/Dijkstra algorithm
type DijkstraInput<'a> = (VertexMap<'a>, EdgeMap<'a>, &'a str, &'a str);

// The output format for the kenny/Dijkstra algorithm
type Route = Vec<String>;

struct RouteMap<'a> {
  vertices: VertexMap<'a>,
  edges: EdgeMap<'a>,
}

impl<'a> RouteMap<'a> {
    pub fn get_dijkstra_route(self, start: &'a str, end: &'a str) -> Output<Route> {
        let client = Client::new(env!("ALGORITHMIA_API_KEY"));
        let dijkstra = Algorithm::new("kenny", "Dijkstra");

        // Declaring type explicitly to enforce valid input types during build
        let input_data: DijkstraInput = (self.vertices, self.edges, start, end);
        println!("Input: {}", json::encode(&input_data));

        let output: Output<Route> = match client.query(dijkstra, &input_data) {
            Ok(out) => out,
            Err(why) => panic!("{}", why),
        };
        output
    }
}

fn main() {
    let mut vertices: VertexMap = HashMap::new();
    let mut edges: EdgeMap = HashMap::new();

    vertices.insert("a", vec![1,1]);
    vertices.insert("b", vec![2,2]);
    vertices.insert("c", vec![3,3]);
    edges.insert("a", vec!["b"]);
    edges.insert("b", vec!["c"]);

    let input_map = RouteMap {
        vertices: vertices,
        edges: edges,
    };

    let output = input_map.get_dijkstra_route("a", "c");
    println!("Shortest route: {}\nCompleted in {} seconds.", output.result, output.duration);
}
