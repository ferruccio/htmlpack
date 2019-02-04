use clap::{
    clap_app, crate_authors, crate_description, crate_name, crate_version, value_t_or_exit,
    values_t, values_t_or_exit, AppSettings,
};
use htmlpack::{PackResult, Packer};
use std::path::{Path, PathBuf};

mod htmlpack;

#[derive(Debug)]
struct Arguments {
    inputs: Vec<PathBuf>,
    outdir: PathBuf,
    search_paths: Vec<PathBuf>,
    overwrite: bool,
}

fn main() {
    let matches = clap_app!(app =>
        (name: crate_name!())
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg INPUT: +required +multiple {file_exists} "HTML source file(s)")
        (@arg path: -p --path +takes_value +multiple {dir_exists}
            "Set search paths for linked files")
        (@arg outdir: -o --("out-dir") +required +takes_value "Set output directory")
        (@arg overwrite: -w --overwrite "Overwrite existing output files")
    )
    .setting(AppSettings::ColoredHelp)
    .setting(AppSettings::UnifiedHelpMessage)
    .get_matches_from(wild::args());

    let arguments = Arguments {
        inputs: values_t_or_exit!(matches, "INPUT", PathBuf),
        outdir: value_t_or_exit!(matches, "outdir", PathBuf),
        search_paths: values_t!(matches, "path", PathBuf).unwrap_or(vec![]),
        overwrite: matches.is_present("overwrite"),
    };
    match process(arguments) {
        Err(e) => println!("{}", e),
        _ => {}
    }
}

fn file_exists(filename: String) -> Result<(), String> {
    if Path::new(&filename).is_file() {
        Ok(())
    } else {
        Err(format!("The file \"{}\" does not exist", filename))
    }
}

fn dir_exists(path: String) -> Result<(), String> {
    if Path::new(&path).is_dir() {
        Ok(())
    } else {
        Err(format!("The directory \"{}\" does not exist", path))
    }
}

fn process(args: Arguments) -> PackResult<()> {
    for input in args.inputs.iter() {
        let mut packer = Packer::new(
            args.outdir.clone(),
            args.search_paths.clone(),
            args.overwrite,
        );
        println!("packing {}", input.to_string_lossy());
        packer.pack(input.clone())?;
    }
    Ok(())
}
