use std::time::Instant;
use rand::{thread_rng, seq::SliceRandom};
use smallvec::SmallVec;
use serde::{Deserialize, Serialize};

use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use nanorand::{Rng, WyRand};
use std::fmt;

use google_cloud_storage::client::Client;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::UploadObjectRequest;
use google_cloud_storage::http::Error;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::sign::SignedURLOptions;

use self::priority_queue::PriorityQueueItem;

mod priority_queue;


#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
struct NodeID(u32);

// implement display options for printing during debug
impl fmt::Display for NodeID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Cost(u16);

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct EdgeWalk {
    to: NodeID,
    cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
struct GraphWalk {
    edges_per_node: HashMap<usize, SmallVec<[EdgeWalk; 4]>>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct EdgePT {
    to: NodeID,
    cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
struct GraphPT {
    edges_per_node: HashMap<usize, SmallVec<[EdgePT; 4]>>,
}


fn main() {
    serialise_list("start_nodes");
    serialise_list("init_travel_times");
    serialise_GraphWalk();

    let start_nodes = read_serialised_vect32("start_nodes");
    let init_travel_times = read_serialised_vect32("init_travel_times");
    let graph_walk = read_GraphWalk();

    println!("graph_walk len: {}", graph_walk.edges_per_node.keys().count());
    println!("start nodes len: {}", start_nodes.len());
    println!("init_travel_times len: {}", init_travel_times.len());

    serialise_GraphWalk();

    
}


fn read_GraphWalk() -> GraphWalk {
    let file = BufReader::new(File::open("serialised_data/p1_main_nodes.bin").unwrap());
    let output: GraphWalk = bincode::deserialize_from(file).unwrap();
    output
}


fn serialise_GraphPT() {
    
}



fn serialise_GraphWalk() {

    let contents = std::fs::read_to_string("data/p1_main_nodes.json").unwrap();

    // check meaning of the '2' in [usize; 2]
    let input: HashMap<usize, Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    // make empty dict
    let mut graph = GraphWalk {
        edges_per_node: HashMap::new(),
    };
    //for i in 0..dict_len {
    //    graph.edges_per_node.insert(i, SmallVec::new());
    //}

    // populate dict
    for (from, input_edges) in input {
        let mut edges = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }

        //let from: usize = from.parse().unwrap();
        //graph.edges_per_node[&from] = edges;
        graph.edges_per_node.insert(from, edges);
    }

    let file = BufWriter::new(File::create("serialised_data/p1_main_nodes.bin").unwrap());
    bincode::serialize_into(file, &graph).unwrap();

    //let dict_len = input.keys().count();
    //println!("serialised p1_main_nodes with len: {}", dict_len);
}



fn serialise_list(filename: &str) {

    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: Vec<i32> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn read_serialised_vect32(filename: &str) -> Vec<i32>{
    let inpath = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(inpath).unwrap());
    let output: Vec<i32> = bincode::deserialize_from(file).unwrap();
    output
}



fn floodfill(graph: &GraphWalk, start: NodeID) -> HashMap<NodeID, Cost> {

    let time_limit = Cost(3600);

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: Cost(0),
        value: start,
    });

    let mut cost_per_node = HashMap::new();

    while let Some(current) = queue.pop() {
        if cost_per_node.contains_key(&current.value) {
            continue;
        }
        if current.cost > time_limit {
            continue;
        }
        cost_per_node.insert(current.value, current.cost);

        /// got some casting here: could any of it be hurting performance?
        for edge in &graph.edges_per_node[&(current.value.0 as usize)] {
            queue.push(PriorityQueueItem {
                cost: Cost(current.cost.0 + edge.cost.0),
                value: edge.to,
            });
        }
    }

    cost_per_node
}


fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}