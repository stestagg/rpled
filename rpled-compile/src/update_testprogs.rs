use colored::Colorize;
use rpled_compile::parser::ParsedProgram;
use rpled_compile::script::ParsedScript;
use std::fs;
use std::path::PathBuf;

fn generate_output_for_file(path: &PathBuf) -> String {
    let mut actual_output = Vec::new();

    let program_result = ParsedProgram::from_file(path);

    match program_result {
        Ok(program) => {
            actual_output.push("Successfully parsed pixelscript".to_string());

            // Try to extract metadata
            match ParsedScript::from_program(program) {
                Ok(script) => {
                    actual_output.push(format!(
                        "Found pixelscript: name='{}', modules={:?}, entrypoint='{}'",
                        script.header.name, script.header.modules, script.header.entrypoint
                    ));
                    actual_output.push("Lua feature check passed".to_string());
                    actual_output.push(format!(
                        "Successfully processed pixelscript '{}'",
                        script.header.name
                    ));
                }
                Err(e) => {
                    actual_output.push(format!("Failed to convert to pixelscript: {}", e));
                }
            }
        }
        Err(e) => {
            actual_output.push(format!("Failed to parse pixelscript file: {}", e));
        }
    }

    actual_output.join("\n")
}

fn main() -> anyhow::Result<()> {
    let test_dir = PathBuf::from("tests/pixelscripts");

    if !test_dir.exists() {
        eprintln!("{}", "Error: tests/pixelscripts directory not found".red());
        std::process::exit(1);
    }

    // Find all .pxl files
    let mut pxl_files: Vec<PathBuf> = fs::read_dir(&test_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "pxl"))
        .collect();

    pxl_files.sort();

    let mut updated_count = 0;

    for pxl_file in &pxl_files {
        let out_file = pxl_file.with_extension("out");
        let file_name = pxl_file.file_name().unwrap().to_string_lossy();

        // Generate the actual output
        let actual_output = generate_output_for_file(pxl_file);

        // Read existing output if it exists
        let existing_output = fs::read_to_string(&out_file).ok();

        // Compare and update if different
        let needs_update = match existing_output {
            Some(existing) => existing.trim() != actual_output.trim(),
            None => true,
        };

        if needs_update {
            fs::write(&out_file, &actual_output)?;
            println!("{} {}", "✓ Updated:".green().bold(), file_name);
            updated_count += 1;
        }
    }

    // Print summary
    println!();
    if updated_count == 0 {
        println!("{}", "All test outputs are up to date.".cyan().bold());
    } else {
        println!(
            "{} {}",
            format!("Updated {} file(s).", updated_count).yellow().bold(),
            "✓".green()
        );
    }

    Ok(())
}
