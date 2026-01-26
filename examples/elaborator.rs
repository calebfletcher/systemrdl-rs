use std::path::Path;

use anyhow::{Context, ensure};

fn main() -> Result<(), anyhow::Error> {
    let arg = std::env::args_os()
        .nth(1)
        .context("no path to an rdl file was provided")?;
    let path = Path::new(&arg);

    ensure!(path.exists(), "path exists");
    let extension = path.extension().context("path has an extension")?;
    ensure!(extension == "rdl", "path points to an rdl file");

    let contents = std::fs::read_to_string(path).context("could not read rdl file")?;
    let ast = systemrdl::parse(&contents)?;

    let elaborated = systemrdl::elaborate(ast)?;

    dbg!(elaborated);

    Ok(())
}
