use botw_editor::SaveData;
use clap::Parser;
use serde_json::{json, Value};
use std::fmt;

use wildmatch::WildMatch;

pub mod hash;
use hash::KEYS;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// game_data.sav input file
    #[arg(short, long)]
    input: String,

    /// name to read, accepts wildcards * and ?
    #[arg(short, long)]
    value: Vec<String>,

    /// set name=value
    #[arg(short, long)]
    set: Vec<String>,

    /// output file
    #[arg(short, long)]
    output: Option<String>,

    /// overwrite the input file
    #[arg(short, long)]
    writeover: bool,

    /// show all values (name, value, hash(name))
    #[arg(short, long)]
    all: bool,
}

#[derive(Copy, Clone, Debug)]
enum Weather {
    Sun = 0,
    Cloudy = 1,
    Rain = 2,
    HeavyRain = 3,
    Snow = 4,
    HeavySnow = 5,
    ThunderStorm = 6,
    ThunderRain = 7,
    BlueSkyRain = 8,
}
impl From<u64> for Weather {
    fn from(value: u64) -> Weather {
        match value {
            0 => Weather::Sun,
            1 => Weather::Cloudy,
            2 => Weather::Rain,
            3 => Weather::HeavyRain,
            4 => Weather::Snow,
            5 => Weather::HeavySnow,
            6 => Weather::ThunderStorm,
            7 => Weather::ThunderRain,
            8 => Weather::BlueSkyRain,
            _ => panic!("unknown weather value"),
        }
    }
}

impl fmt::Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Weather::Sun => "sun",
            Weather::Cloudy => "cloudy",
            Weather::Rain => "rain",
            Weather::HeavyRain => "heavy_rain",
            Weather::Snow => "snow",
            Weather::HeavySnow => "heavy_snow",
            Weather::ThunderStorm => "thunder_storm",
            Weather::ThunderRain => "thunder_rain",
            Weather::BlueSkyRain => "blue_sky_rain",
        };
        write!(f, "{}", s)
    }
}

fn converter(name: &str, value: Value) -> Value {
    if name.starts_with("climateWeather") {
        let mut out = vec![];
        println!("{value}");
        for r in value.as_array().unwrap() {
            let mut tmp = vec![];
            let k = r.as_u64().unwrap();
            for i in 0..6 {
                let w: Weather = (k >> (i * 4) & 0xF).into();
                let s = format!("{w}");
                tmp.push(s);
            }
            out.push(tmp);
        }
        return json!(out);
    }
    value
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
                Ok(v) => println!("{value} {}", converter(value, v)),
                Err(err) => println!("{}", err),
            }
        }
    }

    if args.all {
        for name in KEYS.iter() {
            match s.get(&name) {
                Ok(value) => println!(
                    "{:60} {} {}",
                    name,
                    converter(name, value),
                    s.get_kind(name).unwrap()
                ),
                Err(err) => println!("{} {}", err, name),
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
            Ok(v) => println!("{key} {:?} pre", v),
            Err(err) => println!("{}", err),
        }
        let _out = s.set(key, svalue.clone());
        match s.get(key) {
            Ok(v) => println!("{key} {:?} post", v),
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
