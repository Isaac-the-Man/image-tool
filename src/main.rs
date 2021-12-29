extern crate image;
use image::GenericImageView;
use image::imageops::FilterType;
use image::error::ImageError;

#[macro_use]
extern crate clap;
use clap::App;

use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;
use std::fs;
use std::path::PathBuf;


// TODO: better error handling and hint

#[derive(Debug)]
enum ParseDimensionError {
    ParseInt(ParseIntError),
    BadLen,
}

struct Dimension {
    height: u32,
    width: u32,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.height, self.width)
    }
}

impl FromStr for Dimension {
    type Err = ParseDimensionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sizes_str: Vec<&str> = s.split("x").collect();
        if sizes_str.len() != 2 {
            Err(ParseDimensionError::BadLen)
        } else {
            Ok(Dimension{
                height: sizes_str[0].parse::<u32>().map_err(ParseDimensionError::ParseInt)?,
                width: sizes_str[1].parse::<u32>().map_err(ParseDimensionError::ParseInt)?,
            })
        }
    }
}

#[derive(Debug)]
enum ResizeError {
    ReadImage(ImageError),  // failed to open image
    CriteriaMet,    // dimension alread satisfied
    SaveImage(ImageError),  // failed to save image
}

struct ResizeArguments {
    input: PathBuf,
    output: Option<PathBuf>,
    dimension: Dimension,
    folder: bool,
}

fn main() {
    // parse command line
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    // run commands
    // RESIZE subcommand
    if let Some(resize_matches) = matches.subcommand_matches("resize") {

        // parse arguments and flags
        let args = ResizeArguments {
            input: PathBuf::from(resize_matches.value_of("INPUT").unwrap()),
            output: resize_matches.value_of("output").map(|x| PathBuf::from(x)),
            dimension: Dimension::from_str(resize_matches.value_of("dimension").unwrap()).unwrap(),
            folder: resize_matches.is_present("folder"),
        };

        // check folder tag
        if args.folder {

            // check if INPUT is folder
            if !args.input.is_dir() {
                // not a folder, terminating...
                return println!("'{}' is not a folder. Remove flag '-f' to parse a file.", args.input.display());
            }

            // read files in the folder (shallow read)
            let files: Vec<PathBuf> = read_folder(resize_matches.value_of("INPUT").unwrap());
            // keeps track of failed files
            let mut failures: Vec<String> = Vec::new();

            // resize all files in folder, output here specifies a folder
            for f in files {
                // default output to input (replace)
                let mut output = args.input.clone();
                // check if output is present (copy to new folder) or replace originals
                if let Some(ref temp_output) = args.output {
                    // output is present, save in specified folder
                    output = temp_output.clone();
                    output.push(f.file_name().unwrap());
                }
                // resize and save
                match resize_image(f.to_str().unwrap(), &args.dimension, Some(output.to_str().unwrap())) {
                    Ok(_) => {
                        println!("[PASS] resized '{}'", f.to_str().unwrap());
                    },
                    Err(err) => {
                        match err {
                            ResizeError::ReadImage(_) => {
                                println!("[FAIL] failed to read '{}'", f.to_str().unwrap());
                                failures.push(String::from(f.to_str().unwrap()));
                            },
                            ResizeError::CriteriaMet => {
                                println!("[WARN] size already satisfied for '{}'", f.to_str().unwrap());
                            },
                            ResizeError::SaveImage(_) => {
                                println!("[FAIL] failed to save '{}'", f.to_str().unwrap());
                                failures.push(String::from(f.to_str().unwrap()));
                            },
                        }
                    },
                };
            }

            // list failures
            println!("{} FAILED CASES:", failures.len());
            for failed in failures {
                println!("{}", failed);
            }

        } else {
            // check if input is file
            if !args.input.is_file() {
                // not file, terminating...
                println!("'{}' is not a file. Use flag '-f' to parse a folder.", args.input.display());
            } else {
                // resize a specific image, output here specifies a filename
                resize_image(args.input.to_str().unwrap(), &args.dimension,
                    args.output.map(|x| String::from(x.to_str().unwrap())).as_deref()).unwrap();
            }
        }
    } else {
        // no arguments provided, terminate
        println!("No valid arguments provided, terminating...");
    }
}

// get array of files give folder path
fn read_folder(basepath: &str) -> Vec<PathBuf> {
    let paths = fs::read_dir(basepath).unwrap();
    paths.into_iter().map(|p| p.unwrap().path())
        .filter(|p| p.is_file())
        .collect()
}

// resize a image given path. See enum ResizeError for related errors
fn resize_image(path: &str, d: &Dimension, outpath: Option<&str>) -> Result<(), ResizeError> {
    // read image
    let img = match image::open(path) {
        Ok(val) => val,
        Err(err) => {
            return Err(ResizeError::ReadImage(err));
        },
    };
    // check necessity of resizing
    if img.height() == d.height && img.width() == d.width {
        // size already satisfied, copy (if output is provided) then exit
        if let Some(out) = outpath {
            // output is provided, save as specified
            img.save(out).unwrap();
        }
        return Err(ResizeError::CriteriaMet);
    }
    // resizing
    let resized = img.resize_exact(d.width, d.height, FilterType::Nearest);
    // check if output is provided
    if let Some(out) = outpath {
        // output is provided, save as specified
        match resized.save(out) {
            Ok(_) => Ok(()),
            Err(err) => Err(ResizeError::SaveImage(err)),
        }
    } else {
        // output not provided, replace original file
        match resized.save(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(ResizeError::SaveImage(err)),
        }
    }
}