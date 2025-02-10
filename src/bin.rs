use clap::Parser;
use savefile::{get_hash, SaveData};
use serde_json::Value;

use wildmatch::WildMatch;

pub mod hash;
use hash::KEYS;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    value: Vec<String>,

    #[arg(short, long)]
    set: Vec<String>,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    writeover: bool,

    #[arg(short, long)]
    all: bool,
}

fn main() {
    let args = Args::parse();

    let mut s = SaveData::read(&args.input).unwrap();

    for val in args.value {
        //let re = Regex::new(&val).unwrap();
        let re = WildMatch::new(&val);
        let values: Vec<_> = KEYS.iter().filter(|key| re.matches(key)).collect();
        for value in values {
            match s.get(&value) {
                Ok(v) => println!("{value}: {}", v),
                Err(err) => println!("{}", err),
            }
        }
    }

    if args.all {
        for name in KEYS.iter() {
            match s.get(&name) {
                Ok(value) => println!("{:60} {} {}", name, value, get_hash(name)),
                Err(err) => println!("{}", err),
            }
        }
    }

    if !args.set.is_empty() {
        println!("Setting values ...")
    }
    for val in args.set {
        let vals: Vec<_> = val.split('=').collect();
        let key = vals[0];
        let value = vals[1];
        let svalue: Value = serde_json::from_str(value).unwrap();
        match s.get(key) {
            Ok(v) => println!("{key}: {:?} pre", v),
            Err(err) => println!("{}", err),
        }
        let _out = s.set(key, svalue.clone());
        match s.get(key) {
            Ok(v) => println!("{key}: {:?} post", v),
            Err(_err) => {}
        }
    }
    if args.writeover {
        println!("Writing output to {}...", args.input);
        s.write(&args.input).unwrap();
    } else if let Some(filename) = args.output {
        println!("Writing output to {}...", filename);
        s.write(&filename).unwrap();
    }
}
