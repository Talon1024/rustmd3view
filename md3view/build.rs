use std::{fs, env, error::Error, process::Command, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
	let here = env::current_dir()?;
	let shaders = fs::read_dir(here.join("assets"))?.filter_map(|f| {
		let fname = f.ok()?.path();
		match fname.extension() {
			Some(e) => {
				if e == "vert" || e == "frag" {
					println!("cargo:rerun-if-changed={}", fname.display());
					Some(fname)
				} else {
					None
				}
			},
			None => None
		}
	}).collect::<Box<[PathBuf]>>();
	let mut cmd = Command::new("glslangValidator");
	cmd.args(shaders.into_iter());
	match cmd.status()?.success() {
		true => Ok(()),
		false => {
			let error = String::from_utf8(cmd.output()?.stderr.clone())?;
			Err(format!("Could not compile shaders!\n{error}").into())
		},
	}
}
