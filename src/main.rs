use crate::data::graph::Graph;
use std::time::Instant;

pub mod cpd;
pub mod data;

use clap::Parser;
use cpd::{
    candidate_generation::AlgoCandidateGeneration,
    config::CPDConfig,
    graph_matching::{AlgoGraphMatching, GEDEditCosts},
};

/// Fast Rust implementation for Collaboration Pattern Detection
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file with the graph database
    #[arg(short, long)]
    input: String,

    /// Output file for the resulting subgraphs, if "stdout", the resulting patterns will be printed to the
    /// console after processing finished with ######
    #[arg(short, long, default_value = "stdout")]
    output: String,

    /// Exact support
    #[arg(long, default_value_t = 2)]
    support_exact: usize,

    /// Relaxed support
    #[arg(long, default_value_t = 2)]
    support_relaxed: usize,

    /// Graph matching:
    /// - "cosine" (node and edge vector similarity, uses the alpha parameter),
    /// - "cosine-dyn" (dynamic threshold based on graph size),
    /// - "ged" (approx. graph edit distance)
    /// - "vf2" (only exact matches)
    #[arg(long, default_value = "cosine")]
    graph_matching: String,

    /// Relaxed threshold
    /// - values [0.0..1.0] for graph matching "cosine" (1.0 means exact matches)
    /// - values >= 0 for graph matching "ged" (0 means exact matches)
    #[arg(long, default_value_t = 0.95)]
    relaxed_threshold: f64,

    /// Dynamic threshold minimum (for "cosine-dyn")
    #[arg(long, default_value_t = 0.6)]
    threshold_min: f64,

    /// Dynamic threshold maximum (for "cosine-dyn")
    #[arg(long, default_value_t = 0.9)]
    threshold_max: f64,

    /// Growing factor k (for "cosine-dyn")
    #[arg(long, default_value_t = 0.1)]
    k: f64,

    /// Ignore activity and object nodes.
    /// If set to true, all possible fully connected candidates are created and the activity and
    /// object nodes are ignored.
    #[arg(long, default_value_t = false)]
    ignore_vertex_type: bool,

    /// Activity vertex type.
    /// Parameter is ignored when ignore_vertex_type = True.
    #[arg(long, default_value_t = 0)]
    activity_vertex_type: usize,

    /// Object vertex types
    /// Parameter is ignored when ignore_vertex_type = True.
    #[arg(long, num_args = 0..)]
    object_vertex_types: Vec<usize>,

    /// Minimum number of main vertices (number of activity vertices, or the total number of
    /// vertices when ignore_vertex_type = True)
    #[arg(long, default_value_t = 4)]
    min_vertices: usize,

    /// Maximum number of main vertices (number of activity vertices, or the total number of
    /// vertices when ignore_vertex_type = True)
    #[arg(long, default_value_t = 5)]
    max_vertices: usize,

    /// The alpha value between 0.0 and 1.0 defines the weight importance of the vertex and edge
    /// vectors: if 1.0, the edges are ignored; if 0.0, the vertices are ignored
    #[arg(long, default_value_t = 0.5)]
    alpha: f64,

    /// Suppress debug statements
    #[arg(long, default_value_t = false)]
    silence: bool,
}

fn main() {
    let args = Args::parse();
    let silence = args.silence;

    if !silence {
        println!("-----------------------------------");
        println!("| Collaboration Pattern Detection |");
        println!("-----------------------------------");
    }
    if args.min_vertices > args.max_vertices {
        eprintln!(
            "Parameter error! Min number of activity vertices ({}) > Max number of activity vertices ({})!",
            args.min_vertices, args.max_vertices
        );
        return;
    }
    if args.graph_matching == "cosine" && (args.alpha > 1.0 || args.alpha < 0.0) {
        eprintln!(
            "Parameter error! --alpha should be 0.0 <= alpha <= 1.0, is {}",
            args.alpha
        );
        return;
    }
    if args.graph_matching == "cosine-dyn" && (args.alpha > 1.0 || args.alpha < 0.0) {
        eprintln!(
            "Parameter error! --alpha should be 0.0 <= alpha <= 1.0, is {}",
            args.alpha
        );
        return;
    }
    if args.graph_matching == "cosine"
        && (args.relaxed_threshold > 1.0 || args.relaxed_threshold < 0.0)
    {
        eprintln!(
            "Parameter error! for cosine graph matching, the --relaxed-threshold should be 0.0 <= relaxed_threshold <= 1.0, is {}",
            args.relaxed_threshold
        );
        return;
    }
    if args.graph_matching == "cosine-dyn"
        && (args.threshold_min < 0.0
            || args.threshold_max < 0.0
            || args.threshold_min > args.threshold_max)
    {
        eprintln!(
            "Parameter error! for cosine-dyn graph matching, thresholds should be >= 0 and --threshold-min <= --threshold-max"
        );
        return;
    }
    if args.graph_matching == "ged" && args.relaxed_threshold < 0.0 {
        eprintln!(
            "Parameter error! for ged graph matching, the --relaxed-threshold should be >= 0, is {}",
            args.relaxed_threshold
        );
        return;
    }
    if !silence {
        if args.ignore_vertex_type {
            println!(
                "Note: --ignore-vertex-type is set, so --activity-vertex-type and --object-vertex-types are ignored."
            );
            println!(
                "      Here, --min-vertices / --max-vertices refer to the TOTAL number of vertices in a candidate."
            );
        } else {
            println!(
                "Note: --min-vertices / --max-vertices refer to the number of ACTIVITY vertices in a candidate."
            );
        }
    }

    let now = Instant::now();
    let graphs = Graph::graphs_set_from_file(args.input);
    let graphs = match graphs {
        Ok(ref graphs) => {
            if !silence {
                println!("All good parsing input file, found {} graphs", graphs.len());
            }
            graphs
        }
        Err(err) => {
            eprintln!("Error parsing input file: {}", err);
            return;
        }
    };
    let mut graph_matching = AlgoGraphMatching::GEDFastHungarian {
        edit_costs: GEDEditCosts::default(),
        matching_threshold: args.relaxed_threshold.round() as usize,
    };
    if args.graph_matching == "cosine" {
        graph_matching = AlgoGraphMatching::CosineSimilarity {
            alpha: args.alpha,
            matching_threshold: args.relaxed_threshold,
        }
    } else if args.graph_matching == "cosine-dyn" {
        graph_matching = AlgoGraphMatching::CosineSimilarityDynThreshold {
            alpha: args.alpha,
            matching_threshold_min: args.threshold_min,
            matching_threshold_max: args.threshold_max,
            k: args.k,
        };
    } else if args.graph_matching == "vf2" {
        graph_matching = AlgoGraphMatching::VF2IsomorphismTest;
    }

    let candidate_generation = if args.ignore_vertex_type {
        AlgoCandidateGeneration::FullyConnectedWithOutContext {
            min_number_of_activity_vertices: args.min_vertices,
            max_number_of_activity_vertices: args.max_vertices,
        }
    } else {
        AlgoCandidateGeneration::FullyConnected {
            activity_vertex_type: args.activity_vertex_type,
            object_vertex_types: args.object_vertex_types,
            min_number_of_activity_vertices: args.min_vertices,
            max_number_of_activity_vertices: args.max_vertices,
        }
    };

    let cpd_config = CPDConfig::new(
        candidate_generation,
        graph_matching,
        args.support_exact,
        args.support_relaxed,
        args.silence,
    );
    if !silence {
        println!("Mining patterns..");
    };
    let patterns = cpd_config.run(graphs);
    let delta = now.elapsed().as_millis();
    if !silence {
        println!("Finished. Total time: {delta}ms");
        println!("#######");
    };
    if args.output == "stdout" {
        for g in patterns.iter() {
            println!(
                "{}",
                g.pattern
                    .to_str_repr(Some(g.frequency_exact), Some(g.frequency_relaxed))
            );
        }
    } else {
        use std::fs::File;
        use std::io::Write;

        let file = File::create(&args.output);
        if file.is_err() {
            eprintln!("Failed to create output file");
            return;
        }
        let mut file = file.unwrap();

        for g in patterns.iter() {
            let line = format!(
                "{}\n",
                g.pattern
                    .to_str_repr(Some(g.frequency_exact), Some(g.frequency_relaxed))
            );
            file.write_all(line.as_bytes())
                .expect("Failed to write to output file");
        }
        if !silence {
            println!("Result exported to {}", args.output);
        }
    }
}
