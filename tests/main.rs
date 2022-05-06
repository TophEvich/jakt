use std::{env::temp_dir, io::Write, process::Command};

use jakt::{Compiler, JaktError};

#[test]
fn test_samples() -> Result<(), JaktError> {
    for sample in
        std::fs::read_dir("./samples").expect("if this fails, the repo or harness is broken")
    {
        let sample = sample.expect("if this fails, the filesytem is having issues");

        let path = sample.path();

        if let Some(ext) = path.extension() {
            if ext == "jakt" {
                // Great, we found test file
                let mut output_path = path.clone();
                output_path.set_extension("out");

                if output_path.exists() {
                    // We have an output to compare to, let's do it.

                    let mut compiler = Compiler::new();
                    let cpp_string = compiler.convert_to_cpp(&path)?;

                    let uuid = uuid::Uuid::new_v4();

                    let mut cpp_filename = temp_dir();
                    cpp_filename.push(format!("output{}.cpp", uuid));

                    {
                        let mut cpp_file = std::fs::File::create(&cpp_filename)?;
                        cpp_file.write_all(cpp_string.as_bytes())?;
                    }

                    println!("Test: {:?}", path);
                    let pwd = std::env::current_dir()?;

                    #[cfg(windows)]
                    {
                        let mut exe_name = temp_dir();
                        exe_name.push(format!("output{}.exe", uuid));

                        let mut obj_name = temp_dir();
                        obj_name.push(format!("output{}.obj", uuid));

                        let status = Command::new("cl.exe")
                            .arg(&cpp_filename)
                            .arg("/I")
                            .arg(pwd)
                            .arg("/Fo:")
                            .arg(&obj_name)
                            .arg("/Fe:")
                            .arg(&exe_name)
                            .arg("/std:c++20")
                            .status()?;
                        assert!(status.success());

                        let binary_run = Command::new(&exe_name).output()?;

                        let binary_output = String::from_utf8_lossy(&binary_run.stdout).to_string();
                        let binary_output = binary_output.replace("\r\n", "\n");

                        let baseline_text = std::fs::read_to_string(&output_path)?;
                        let baseline_text = baseline_text.replace("\r\n", "\n");

                        assert_eq!(
                            binary_output,
                            baseline_text,
                            "\r\nTest: {}",
                            path.to_string_lossy()
                        );
                    }
                    #[cfg(not(windows))]
                    {
                        compiler = "clang"
                    }
                }
            }
        }
    }

    Ok(())
}
