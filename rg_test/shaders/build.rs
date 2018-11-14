extern crate fxc;

const ASSET_SOURCE_DIR: &'static str = "shaders\\";
const ASSET_ARTIFACT_DIR: &'static str = "shaders\\";

enum Options<'a> {
    Simple(&'a str, &'a str, &'a str, &'a str)
}

fn compile_hlsl(fxc: &fxc::Fxc, options: &[Options]) -> () {
    let mut paths: Vec<String> = Vec::with_capacity(options.len());

    for option in options {
        match option {
            &Options::Simple(name, entry, profile, output) => {
                let output_path = format!("{}{}", ASSET_ARTIFACT_DIR, output);
                let input_path = format!("{}{}", ASSET_SOURCE_DIR, name);

                let cmd = [
                    "/nologo",
                    "/T", profile,
                    "/E", entry,
                    "/Fo", &output_path,
                    &input_path
                ];

                println!("fxc.exe {}", cmd.join(" "));
                let output = fxc.run(cmd.iter()).unwrap();
                if !output.status.success() {
                    panic!("{}", String::from_utf8(output.stderr).unwrap());
                } else {
                    print!("{}", String::from_utf8(output.stdout).unwrap());
                }

                paths.push(input_path.to_string());
                paths.push(output_path.to_string());
            }
        }

    }

    for path in paths {
        println!("cargo:rerun-if-changed={}", path);
    }
}

fn main() {
    let fxc = fxc::Fxc::new().unwrap();

    compile_hlsl(&fxc, &[
        Options::Simple("simple.hlsl", "VS", "vs_5_0", "vs.o"),
        Options::Simple("simple.hlsl", "PS", "ps_5_0", "ps.o")
    ]);
}
