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
use std::path::{Path, PathBuf};


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
    Image(ImageError)
}

fn main() {
    // parse command line
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    // run commands
    // RESIZE subcommand
    if let Some(resize_matches) = matches.subcommand_matches("resize") {
        // parse dimension
        let dimen = Dimension::from_str(resize_matches.value_of("dimension").unwrap()).unwrap();
        // check folder tag
        if resize_matches.is_present("folder") {
            // check if INPUT is folder
            if !Path::new(resize_matches.value_of("INPUT").unwrap()).is_dir() {
                // not a folder, terminating...
                return println!("'{}' is not a folder. Remove flag '-f' to parse a file.", resize_matches.value_of("INPUT").unwrap());
            }
            // resize all files in folder, output here specifies a folder
            let files: Vec<PathBuf> = read_folder(resize_matches.value_of("INPUT").unwrap());
            // keeps track of failed files
            let mut failures: Vec<String> = Vec::new();
            for f in files {
                if resize_matches.is_present("output") {
                    // output is present, save in specified folder
                    let mut output = PathBuf::from(resize_matches.value_of("output").unwrap());
                    output.push(f.file_name().unwrap());
                    // resize and record failure cases
                    match resize_image(f.to_str().unwrap(), &dimen, Some(output.to_str().unwrap())) {
                        Ok(_) => (),
                        Err(_) => failures.push(String::from(f.to_str().unwrap())),
                    };
                } else {
                    // output is not present, replace file
                    match resize_image(f.to_str().unwrap(), &dimen, resize_matches.value_of("output")) {
                        Ok(_) => (),
                        Err(_) => failures.push(String::from(f.to_str().unwrap())),
                    };
                }
            }
            // list failures
            println!("[{}] Failed Cases:", {failures.len()});
            for failed in failures {
                println!("{}", failed);
            }
        } else {
            // check if input is file
            if !Path::new(resize_matches.value_of("INPUT").unwrap()).is_file() {
                // not file, terminating...
                println!("'{}' is not a file. Use flag '-f' to parse a folder.", resize_matches.value_of("INPUT").unwrap());
            } else {
                // resize a specific image, output here specifies a filename
                resize_image(resize_matches.value_of("INPUT").unwrap(), &dimen, resize_matches.value_of("output")).unwrap();
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

// resize a image given path
fn resize_image(path: &str, d: &Dimension, outpath: Option<&str>) -> Result<(), ResizeError> {
    println!("Resizing image... {}", path);
    // read image
    let img = match image::open(path) {
        Ok(val) => val,
        Err(err) => {
            println!("Failed to open image...");
            return Err(ResizeError::Image(err));
        },
    };
    // check necessity of resizing
    if img.height() == d.height && img.width() == d.width {
        // size already satisfied, exit
        println!("Image is already of size {}", d);
        if let Some(out) = outpath {
            // output is provided, save as specified
            img.save(out).unwrap();
        } else {
            // output not provided, replace original file
            img.save(path).unwrap();
        }
        return Ok(());
    }
    // resizing
    let resized = img.resize_exact(d.width, d.height, FilterType::Nearest);
    // check if output is provided
    if let Some(out) = outpath {
        // output is provided, save as specified
        resized.save(out).unwrap();
    } else {
        // output not provided, replace original file
        resized.save(path).unwrap();
    }
    Ok(())
}