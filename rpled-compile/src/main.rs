use clap::Parser;
use std::path::PathBuf;

mod parser;
mod script;

use parser::ParsedLua;
use script::ParsedScript;

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

    /// Print the compiled bytecode in human-readable format
    #[arg(long)]
    dump: bool,

    /// Dump the Lua AST after parsing and exit
    #[arg(long)]
    dump_lua_ast: bool,

    /// Target VM memory size in KB (4, 8, or 16)
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

    // Validate memory size
    if ![4, 8, 16].contains(&args.memory_size) {
        log::error!("Invalid memory size: {}KB. Must be 4, 8, or 16.", args.memory_size);
        std::process::exit(1);
    }

    log::info!("Target VM memory size: {}KB", args.memory_size);
    log::info!("Dump bytecode: {}", args.dump);

    // Parse the Lua file
    let lua = match ParsedLua::from_file(&args.input) {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to parse Lua file: {:#}", e);
            std::process::exit(1);
        }
    };

    // If dump-lua-ast is set, output the AST and exit
    if args.dump_lua_ast {
        println!("{:#?}", lua.ast);
        std::process::exit(0);
    }

    // Convert to ParsedScript (extracts header and checks for unsupported features)
    let script = match ParsedScript::from_lua(lua) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to convert to pixelscript: {:#}", e);
            std::process::exit(1);
        }
    };

    log::info!("Successfully processed pixelscript '{}'", script.header.name);

    // TODO: Implement compilation to bytecode
    log::warn!("Bytecode generation not yet implemented");
}
