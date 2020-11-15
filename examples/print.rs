use redis_parser::resp2::parse as parse2;
use redis_parser::resp3::parse as parse3;

use clap::arg_enum;
use std::path::PathBuf;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum RespType {
        RESP2,
        RESP3,
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "print_resp", about = "Print RESP output.")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, possible_values = & RespType::variants(), case_insensitive = true, default_value = "resp2")]
    version: RespType,
}

fn main() {
    let args: Opt = Opt::from_args();
    let raw_text = std::fs::read_to_string(args.input).expect("File does not exist");
    let replaced = raw_text.replace('\n', "\r\n");
    let mut data = replaced.as_bytes();

    // I don't know how to generalize this :(
    match args.version {
        RespType::RESP2 => {
            let it = std::iter::from_fn(move || match parse2(data) {
                Ok((i, o)) => {
                    data = i;
                    Some(o)
                }
                _ => None,
            });

            for item in it {
                println!("{:?}", item);
            }
        }
        RespType::RESP3 => {
            let it = std::iter::from_fn(move || match parse3(data) {
                Ok((i, o)) => {
                    data = i;
                    Some(o)
                }
                _ => None,
            });

            for item in it {
                println!("{:?}", item);
            }
        }
    };
}
