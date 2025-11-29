use crate::data::graph::Graph;
use core::f32;
use std::time::Instant;

pub mod cpd;
pub mod data;

use clap::Parser;
use cpd::{
    candidates::AlgoCandidateGeneration, config::CPDConfig, pattern_matching::AlgoPatternMatching,
};

/// Fast Rust implementation for gSpan
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file with the graph database
    #[arg(short, long)]
    input: String,

    /// Output file for the resulting subgraphs
    #[arg(short, long, default_value = "out.txt")]
    output: String,

    /// Min exact support
    #[arg(long, default_value_t = 2)]
    support_exact: usize,

    /// Min relaxed support
    #[arg(long, default_value_t = 2)]
    support_relaxed: usize,

    /// Relaxed threshold
    #[arg(long, default_value_t = 0.0f32)]
    relaxed_threshold: f32,

    /// Activity vertex type
    #[arg(long, default_value_t = 0)]
    activity_node_type: usize,

    /// Object nodes vertex types
    #[arg(long, num_args = 0..)]
    object_node_types: Vec<usize>,

    /// Minimum number of main vertices
    #[arg(long, default_value_t = 1)]
    min_vertices: usize,

    /// Maximum number of the main vertices
    #[arg(long, default_value_t = 10)]
    max_vertices: usize,
}

fn main() {
    let args = Args::parse();

    println!("CPD Subgraph Mining");
    println!("---------------------");
    println!("Using arguments:");
    println!("{:?}", args);
    let _now = Instant::now();
    let graphs = Graph::graphs_set_from_file(args.input);
    match graphs {
        Ok(ref graphs) => {
            println!("All good parsing input file, found {} graphs", graphs.len());
        }
        Err(err) => panic!("{}", err.to_string()),
    }
    let _graphs = graphs.unwrap();
    let cpd_config = CPDConfig::new(
        AlgoCandidateGeneration::FullyConnected {
            activity_node_type: args.activity_node_type,
            object_node_types: args.object_node_types,
        },
        AlgoPatternMatching::CosineSimilarity { alpha: 0.5f32 },
    );

    cpd_config.run(&_graphs);
    println!("Mining subgraphs..");
    todo!();
    let delta = _now.elapsed().as_millis();
    println!("Finished.");
    // println!("Found {} subgraphs", subgraphs);
    println!("Took {}ms", delta);
}
