use clap::{App, AppSettings, Arg};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand_core::{RngCore, SeedableRng};
use serde::Deserialize;
use shargrila_chain::*;
use std::collections::VecDeque;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref RND: Mutex<rand_xorshift::XorShiftRng> =
        Mutex::new(rand_xorshift::XorShiftRng::seed_from_u64(RAND_SEED));
}

mod environment;
mod parameters;
mod simulator;
mod transaction_record;
mod user;
mod user_graph;
use environment::*;
use parameters::*;
use simulator::*;
use transaction_record::*;
use user::*;
use user_graph::*;

fn main() {
    let arg_matches = {
        App::new("Shargri-La")
            .version("v0.1.0")
            .about("Sharded blockchain simulator")
            .setting(AppSettings::ColoredHelp)
            .arg(
                Arg::with_name("BIG_QUERY_CSV")
                    .long("csv")
                    .help("A BigQuery Ethereum transactions csv file")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("END_SLOT")
                    .long("end_slot")
                    .help("The number of slots")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("USER_NUM")
                    .long("user_num")
                    .help("The maximum number of users")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("PERCENTAGE_OF_MINIMUM")
                    .long("percentage_of_minimum")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("PERCENTAGE_OF_WEIGHTED_RANDOM")
                    .long("percentage_of_weighted_random")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("PERCENTAGE_OF_DECREASING_MINIMUM")
                    .long("percentage_of_decreasing_minimum")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("POPULAR_USER_EXISTS")
                    .long("popular_user_exists")
                    .help("Whether or not there ia a popular user"),
            )
            .arg(
                Arg::with_name("POPULAR_USER_IS_SWITCHER")
                    .long("popular_user_is_switcher")
                    .help("Whether or not the popular user is a switcher"),
            )
            .arg(
                Arg::with_name("OUTPUT_DIR_PATH")
                    .long("output_dir_path")
                    .help("The path of the output directory")
                    .takes_value(true),
            )
            .get_matches()
    };
    println!("Hello, Shargri-La!");

    let end_slot = if let Some(end_slot) = arg_matches.value_of("END_SLOT") {
        end_slot
            .parse()
            .expect("END_SLOT must be a positive integer")
    } else {
        DEFAULT_END_SLOT
    };
    let mut sim = Simulator::new(end_slot);
    sim.run(&arg_matches);
}
