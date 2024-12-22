use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    file: String,

}

fn main() {
    let args = Args::parse();

    println!("Hello {}!", args.file);

    let path = Path::new(&args.file);
}

fn ifJsonFile(path : &Path) -> bool {
    let mut result : bool = false;

    // file check
    if(!path.is_file()) {
        println!("This is not a file");
        return result;
    }
    // json check
    let json: Result<Value, serde_json::Error> = from_reader::<BufReader<File>, Value>(reader);
    let json = match json {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to parse json.");
            return Err(Box::new(e));
        }
    };
    result = true;
    return result;

}