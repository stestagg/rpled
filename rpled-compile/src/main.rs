use clap::Parser;
use std::path::PathBuf;

use rpled_pixelscript::parse_program;
use rpled_pixelscript::error::format_errors;
use rpled_pixelscript::format::AstFormat as _;

#[derive(Parser, Debug)]
#[command(
    name = "rpled-compile",
    version,
    about = "Pixelscript compiler - Compiles pixelscript (.pxl) files to RPLed bytecode",
    long_about = "A compiler that translates high-level pixelscript control scripts (Lua subset) \
                  into bytecode for the RPLed interpreter running on RP2XXX microcontrollers."
)]
struct Args {
    /// Input pixelscript file (.pxl)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file (defaults to <input>.bin)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Increase verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long)]
    quiet: bool,

    /// Dump the AST after parsing and exit
    #[arg(long)]
    dump_ast: bool,

    /// Target VM memory size in KB
    #[arg(long, value_name = "KB", default_value = "8")]
    memory_size: u16,
}

fn main() {
    let args = Args::parse();

    // Initialize logger based on verbosity
    let log_level = if args.quiet {
        "error"
    } else {
        match args.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();

    log::info!("RPLed Pixelscript Compiler v{}", env!("CARGO_PKG_VERSION"));
    log::info!("Input file: {}", args.input.display());

    // Determine output file
    let output = args.output.unwrap_or_else(|| {
        let mut out = args.input.clone();
        out.set_extension("bin");
        out
    });
    log::info!("Output file: {}", output.display());

    // Validate input file exists and has .pxl extension
    if !args.input.exists() {
        log::error!("Input file does not exist: {}", args.input.display());
        std::process::exit(1);
    }

    if args.input.extension().and_then(|s| s.to_str()) != Some("pxl") {
        log::warn!(
            "Input file does not have .pxl extension: {}",
            args.input.display()
        );
    }

    log::info!("Target VM memory size: {}KB", args.memory_size);

    // Parse the pixelscript file
    let program = match parse_program(&std::fs::read_to_string(&args.input).unwrap()).into_result() {
        Ok(p) => p,
        Err(e) => {
            format_errors(
                &std::fs::read_to_string(&args.input).unwrap(),
                &args.input.to_string_lossy(),
                e,
            );
            std::process::exit(1);
        }
    };

    // If dump-ast is set, output the AST and exit
    if args.dump_ast {
        let mut formatter = rpled_pixelscript::format::Formatter::new(
            rpled_pixelscript::format::FormatOptions::new(2)
        );
        program.format_into(&mut formatter);
        println!("{}", formatter.into_string());
        std::process::exit(0);
    }
    
    

    // // Extract pixelscript metadata
    // let script = match ParsedScript::from_program(program) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         log::error!("Failed to extract metadata: {:#}", e);
    //         std::process::exit(1);
    //     }
    // };

    // log::info!("Successfully processed pixelscript '{}'", script.header.name);

    // // TODO: Implement compilation to bytecode
    // log::warn!("Bytecode generation not yet implemented");
}
