use rpled_compile::parser::ParsedLua;
use rpled_compile::script::ParsedScript;
use rstest::*;
use std::path::PathBuf;

#[rstest]
fn test_pixelscripts(#[files("tests/pixelscripts/*.pxl")] path: PathBuf) {
    // Read expected output
    let expected_file = path.with_extension("out");
    let expected_output = std::fs::read_to_string(&expected_file)
        .unwrap_or_else(|_| panic!("Missing .out file for {:?}", path));

    // Parse the Lua file
    let mut actual_output = Vec::new();

    let lua_result = ParsedLua::from_file(&path);

    match lua_result {
        Ok(lua) => {
            actual_output.push("Successfully parsed pixelscript".to_string());

            // Try to convert to ParsedScript
            match ParsedScript::from_lua(lua) {
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
            actual_output.push(format!("Failed to parse Lua file: {}", e));
        }
    }

    let actual = actual_output.join("\n");

    assert_eq!(
        actual.trim(),
        expected_output.trim(),
        "Output did not match for test {:?}",
        path
    );
}
