extern crate image;
use image::GenericImageView;
use image::imageops::FilterType;
use image::error::ImageError;
use image::ImageFormat;

#[macro_use]
extern crate clap;
use clap::App;

use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;
use std::fs;
use std::path::{Path, PathBuf};
use std::convert::AsRef;

// TODO: convert


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
enum ParseFormatError {
    UnknownFormat,
}

#[derive(Debug)]
enum ResizeError {
    ReadImage(ImageError),  // failed to open image
    CriteriaMet,    // dimension alread satisfied
    SaveImage(ImageError),  // failed to save image
}

#[derive(Debug)]
enum ConvertError {
    ReadImage(ImageError),
    CriteriaMet,
    Convert,    // error while converting
    SaveImage(ImageError),
}

struct ResizeArguments {
    input: PathBuf,
    output: Option<PathBuf>,
    dimension: Dimension,
    folder: bool,
}

struct ConvertArguments {
    input: PathBuf,
    output: Option<PathBuf>,
    format: ImageFormat,
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
                let mut output = f.clone();
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

        // CONVERT subcommands
    } else if let Some(con_matches) = matches.subcommand_matches("convert") {

        // parse arguments and flags
        let args = ConvertArguments {
            input: PathBuf::from(con_matches.value_of("INPUT").unwrap()),
            output: con_matches.value_of("output").map(|x| PathBuf::from(x)),
            format: ImageFormat::from_extension(con_matches.value_of("format").unwrap()).unwrap(),
            folder: con_matches.is_present("folder"),
        };

        // check folder tag
        if args.folder {
            unimplemented!()
        } else {
            // check if input is file
            if !args.input.is_file() {
                // not file, terminating...
                println!("'{}' is not a file. Use flag '-f' to parse a folder.", args.input.display());
            } else {
                // resize a specific image, output here specifies a filename
                convert_image(args.input, args.format, args.output).unwrap();
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
            match img.save(out) {
                Ok(_) => (),
                Err(err) => {return Err(ResizeError::SaveImage(err));}
            };
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

// convert image to another format
fn convert_image<T: AsRef<Path>>(path: T, format: ImageFormat, outpath: Option<T>) -> Result<(), ConvertError> {
    // check necessity of converting by checking file extension
    if format.extensions_str().contains(&path.as_ref().extension().unwrap().to_str().unwrap()) {
        // same format, terminate
        return Err(ConvertError::CriteriaMet);
    }
    // read image
    let img = match image::open(path.as_ref()) {
        Ok(val) => val,
        Err(err) => {
            return Err(ConvertError::ReadImage(err));
        },
    };
    // check if output is provided, convert and save
    if let Some(out) = outpath {
        // output is provided, save as specified
        match img.save_with_format(out.as_ref(), format) {
            Ok(_) => Ok(()),
            Err(err) => Err(ConvertError::SaveImage(err)),
        }
    } else {
        // output not provided, replace original file
        match img.save_with_format(path, format) {
            Ok(_) => Ok(()),
            Err(err) => {
                match err {
                    ImageError::Encoding(_) => Err(ConvertError::Convert),
                    ImageError::Decoding(_) => Err(ConvertError::SaveImage(err)),
                    ImageError::Parameter(_) => Err(ConvertError::SaveImage(err)),
                    ImageError::Limits(_) => Err(ConvertError::SaveImage(err)),
                    ImageError::Unsupported(_) => Err(ConvertError::SaveImage(err)),
                    ImageError::IoError(_) => Err(ConvertError::SaveImage(err)),
                }
            },
        }
    }
}