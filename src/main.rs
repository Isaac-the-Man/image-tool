extern crate image;
use image::GenericImageView;
use image::imageops::FilterType;
use image::error::ImageError;
use image::ImageFormat;

#[macro_use]
extern crate clap;
use clap::App;

use text_io::read;

use std::fmt;
use std::str::FromStr;
use std::num::ParseIntError;
use std::fs;
use std::path::{Path, PathBuf};
use std::convert::AsRef;

// TODO: rewrite resize as AsRef

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
    ParseFormat(ParseFormatError),
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
    yes: bool,
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
            let mut failures: Vec<PathBuf> = Vec::new();

            // resize all files in folder, output here specifies a folder
            for f in &files {
                // default output to input (replace)
                let mut output = f.clone();
                // check if output is present (copy to new folder) or replace originals
                if let Some(ref temp_output) = args.output {
                    // output is present, save in specified folder
                    output = temp_output.clone();
                    output.push(f.file_name().unwrap());
                }
                // resize and save
                match resize_image(f, &args.dimension, Some(&output)) {
                    Ok(_) => {
                        println!("[PASS] resized '{}'", f.display());
                    },
                    Err(err) => {
                        match err {
                            ResizeError::ReadImage(_) => {
                                println!("[FAIL] failed to read '{}'", f.display());
                                failures.push(f.to_owned());
                            },
                            ResizeError::CriteriaMet => {
                                println!("[WARN] size already satisfied for '{}'", f.display());
                            },
                            ResizeError::SaveImage(_) => {
                                println!("[FAIL] failed to save '{}'", f.display());
                                failures.push(f.to_owned());
                            },
                        }
                    },
                };
            }

            // list failures
            println!("{} FAILED CASES:", failures.len());
            failures.into_iter().for_each(|x| println!("{}", x.display()));

        } else {
            // check if input is file
            if !args.input.is_file() {
                // not file, terminating...
                println!("'{}' is not a file. Use flag '-f' to parse a folder.", args.input.display());
            } else {
                // resize a specific image, output here specifies a filename
                resize_image(args.input, &args.dimension, args.output).unwrap();
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
            yes: con_matches.is_present("yes"),
        };

        // check folder tag
        if args.folder {
            
            // check if INPUT is folder
            if !args.input.is_dir() {
                // not a folder, terminating...
                return println!("'{}' is not a folder. Remove flag '-f' to parse a file.", args.input.display());
            }

            // read files in the folder (shallow read)
            let files: Vec<PathBuf> = read_folder(con_matches.value_of("INPUT").unwrap());
            // keeps track of failed files
            let mut failures: Vec<PathBuf> = Vec::new();

            // resize all files in folder, output here specifies a folder
            for f in &files {
                // default output to input (replace)
                let mut output = f.clone();
                // check if output is present (copy to new folder) or replace originals
                if let Some(ref temp_output) = args.output {
                    // output is present, save in specified folder
                    output = temp_output.clone();
                    // construct target path with specified format
                    let mut f_clone = f.clone();
                    f_clone.set_extension(con_matches.value_of("format").unwrap());
                    output.push(f_clone.file_name().unwrap());
                } else {
                    output.set_extension(con_matches.value_of("format").unwrap());
                }
                // resize and save
                match convert_image(f, args.format, &output) {
                    Ok(_) => {
                        println!("[PASS] converted '{}'", f.to_str().unwrap());
                    },
                    Err(err) => {
                        match err {
                            ConvertError::ReadImage(_) => {
                                println!("[FAIL] failed to read '{}'", f.display());
                                failures.push(f.to_owned());
                            },
                            ConvertError::CriteriaMet => {
                                println!("[WARN] format already satisfied '{}'", f.display());
                            },
                            ConvertError::Convert => {
                                println!("[FAIL] failed to convert '{}'", f.display());
                                failures.push(f.to_owned());
                            },
                            ConvertError::SaveImage(_) => {
                                println!("[FAIL] failed to save '{}'", f.display());
                                failures.push(f.to_owned());
                            },
                            ConvertError::ParseFormat(_) => {
                                println!("[FAIL] Failed to parse format of '{}'", f.display());
                                failures.push(f.to_owned());
                            },
                        }
                    },
                };
            }

            // list failures
            println!("{} FAILED CASES:", failures.len());
            failures.into_iter().for_each(|x| println!("{}", x.display()));

            // ask to replace originals
            if args.output.is_none() {
                if args.yes || ask_to_remove_files() {
                    delete_files(&files);
                }
            }
        } else {
            // check if input is file
            if !args.input.is_file() {
                // not file, terminating...
                println!("'{}' is not a file. Use flag '-f' to parse a folder.", args.input.display());
            } else {
                // resize a specific image, output here specifies a filename
                // default output to input (replace)
                let mut output = args.input.clone();
                output.set_extension(con_matches.value_of("format").unwrap());
                // check if output is present (copy to new folder) or replace originals
                if let Some(ref temp_output) = args.output {
                    // output is present, save in specified folder
                    output = temp_output.clone();
                }
                // resize and save
                convert_image(&args.input, args.format, &output).unwrap();
                // ask to replace originals
                if args.output.is_none() {
                    if args.yes || ask_to_remove_files() {
                        delete_files(&vec![&args.input]);
                    }
                }
            }
        }

    } else {
        // no arguments provided, terminate
        println!("No valid arguments provided, terminating...");
    }
}

// get array of files give folder path
fn read_folder<T: AsRef<str>>(basepath: T) -> Vec<PathBuf> {
    let paths = fs::read_dir(basepath.as_ref()).unwrap();
    paths.into_iter().map(|p| p.unwrap().path())
        .filter(|p| p.is_file())
        .collect()
}

// resize a image given path. See enum ResizeError for related errors
fn resize_image<T: AsRef<Path>>(path: T, d: &Dimension, outpath: Option<T>) -> Result<(), ResizeError> {
    // read image
    let img = match image::open(path.as_ref()) {
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
            match img.save(out.as_ref()) {
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
        match resized.save(path.as_ref()) {
            Ok(_) => Ok(()),
            Err(err) => Err(ResizeError::SaveImage(err)),
        }
    }
}


// ask to remove original files
fn ask_to_remove_files() -> bool {
    loop {
        println!("Do you wished to remove the original files (y/n)?");
        let res: String = read!("{}\n");
        if res.trim().eq("y") || res.trim().eq("n") {
            return res.trim().eq("y");
        }
    }
}

// remove files
fn delete_files<T: AsRef<Path>>(files: &[T]) {
    // removing files
    for f in files {
        match fs::remove_file(f.as_ref()) {
            Ok(_) => {
                println!("[PASS] deleted '{}'", f.as_ref().display());
            },
            Err(_) => {
                println!("[FAIL] failed to delete '{}'", f.as_ref().display());
            },
        }
    }
}

// convert image to another format
fn convert_image<T: AsRef<Path>>(path: T, format: ImageFormat, outpath: T) -> Result<(), ConvertError> {
    // check necessity of converting by checking file extension
    let input_format = match ImageFormat::from_path(path.as_ref()) {
        Ok(f) => f,
        Err(_) => {return Err(ConvertError::ParseFormat(ParseFormatError::UnknownFormat));}
    };
    if input_format == format {
        // same format as specified, don't convert
        return Err(ConvertError::CriteriaMet);
    }
    // read image
    let img = match image::open(path.as_ref()) {
        Ok(val) => val,
        Err(err) => {
            return Err(ConvertError::ReadImage(err));
        },
    };
    // convert and save
    match img.save_with_format(outpath, format) {
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