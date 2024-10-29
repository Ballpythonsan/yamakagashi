use std::path::PathBuf;
use clap::{self, Arg, Command};
use yamakagashi::{do_encode, do_decode};

/*{
#[derive(Parser, Debug)]
struct Cli {
    #[clap(long)]
    command: Opt,
    #[clap(short = 'q', long = "qualityquality value_name = "target_sqr", default_value_t = 85)]
    quality: i32,
    input_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
}

#[derive(Debug)]
enum Opt {
    encode,
    decode,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    encode,
    decode,
} }*/

fn main() {
    
    /* env analyze
    command e.g. 
    $ yamakagashi encode target_sqR xxx.bmp xxx.yama
    $ yamakagashi decode xxx.yama xxx.bmp
    $ yamakagashi help
    $ yamakagashi version
    */
    let matches = Command::new("yamakagashi")
        .subcommand(
            Command::new("encode")
                .arg(Arg::new("input_path").required(true).index(1).value_parser(clap::value_parser!(PathBuf)))
                .arg(Arg::new("output_path").required(false).index(2).value_parser(clap::value_parser!(PathBuf)))
                .arg(Arg::new("quality").required(false).index(3).value_parser(clap::value_parser!(i32).range(0..=100)))
            )
        .subcommand(
            Command::new("decode")
                .arg(Arg::new("input_path").required(true).index(1).value_parser(clap::value_parser!(PathBuf)))
                .arg(Arg::new("output_path").required(false).index(2).value_parser(clap::value_parser!(PathBuf)))
            )
        .get_matches();

    /*{let input_path = matches.get_one::<PathBuf>("input_path").unwrap();
    let temp_path = PathBuf::from(input_path.file_name().unwrap()).with_extension("yama");
    let output_path = match matches.get_one::<PathBuf>("output_path") {
        Some(output_path) => output_path,
        _ => &temp_path,
    };}*/
    
    
    let result = match matches.subcommand() {
        Some(("encode", matches)) => {
            let input_path = matches.get_one::<PathBuf>("input_path").unwrap();
            let output_path = match matches.get_one::<PathBuf>("output_path") {
                Some(output_path) => output_path,
                _ => &PathBuf::from(input_path.file_name().unwrap()).with_extension("yama"),
            };
            const DEFAULT_QUALITY: i32 = 50;
            let quality = match matches.get_one::<i32>("quality") {
                Some(quality) => *quality,
                _ => DEFAULT_QUALITY
            };
            do_encode(input_path, output_path, quality)
        }


        Some(("decode", matches)) => {
            let input_path = matches.get_one::<PathBuf>("input_path").unwrap();
            let output_path = match matches.get_one::<PathBuf>("output_path") {
                Some(output_path) => output_path,
                _ => &PathBuf::from(input_path.file_name().unwrap()).with_extension("bmp"),
            };
            do_decode(input_path, output_path)},

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    match result {
        Ok(()) => println!("Successful"),
        Err(why) => println!("Failed! reason of {}", why),
    }
    
    
    // encording

    //call lib.rs/encording
    // done encording message

    // decording
    // done decording message

    // done message

    // terminate
}