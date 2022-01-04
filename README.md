# image-tool
A simple mass image manipulation commandline tool for resizing and converting format. This tool is specifically designed to for performing conversion on large amount of images with different formats efficienctly.

## Usage
Currently there are two subcommands: `resize` and `convert`
- `resize`: resize image(s), can be a file or a folder
- `convert`: convert image(s) to a certain format, works with a file or a folder.

### Usage for `resize`
```
USAGE:
    image-tool.exe resize [FLAGS] [OPTIONS] <INPUT> --dimension <dimension>

FLAGS:
    -f, --folder     Perform resize for all images in a folder
    -g, --guess      Guess file format based on the first few bytes
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --dimension <dimension>    Specify the output dimension of the file in the form "SIZExSIZE" (eg. "64x64")
    -o, --output <output>          Specify the output file name and path

ARGS:
    <INPUT>    Specify the path of the file/folder to perform the operation
```
An example to resize all images under folder `test/` to size 36x36. Notice all resized images will be replaced in place.
```
image-tool resize test/ -f --dimension 36x36
```
To resize a file `test.png` to 128x256, and save it as a new file `resized.png` without replacing the original.
```
image-tool resize test.png --dimension 128x256 -o resized.png
```
**Note**: By default image format are extracted from the file extension. Adding the flag `-g` will force the tool to guess the format based on the first few magic bytes, while sacrificing some efficiency.

### Usage for `convert`
```
USAGE:
    image-tool.exe convert [FLAGS] [OPTIONS] <INPUT> --format <format>

FLAGS:
    -f, --folder     Perform resize for all images in a folder
    -g, --guess      Guess file format based on the first few bytes
    -h, --help       Prints help information
    -V, --version    Prints version information
    -y, --yes        Agrees to all following prompts (eg. delete original files)

OPTIONS:
    -F, --format <format>    Specify the output format (eg. "PNG"). Supported format are PNG, JPEG
    -o, --output <output>    Specify the output file name and path

ARGS:
    <INPUT>    Specify the path of the file/folder to perform the operation
```
An example to converting all images under folder `test/` to the format `jpeg`, and save it to another folder `result/`. A prompt will ask if the original files should be deleted or not.
```
image-tool convert test/ -f --format jpeg -o result/
```
To convert a single file `test.jpeg` to a `png`, and proceeds to delete the original file without a prompt.
```
image-tool convert test.jpeg --format png -y
```
**Note**: By default image format are extracted from the file extension. Adding the flag `-g` will force the tool to guess the format based on the first few magic bytes, while sacrificing some efficiency.

## build
Building the tool from scratch requires the `rust` compiler and `cargo` to be installed on your machine. Then execute the following commands:
```
git clone https://github.com/Isaac-the-Man/image-tool.git
cd image-tool/
cargo build --release
```
Next to be able to use the tool anywhere on your machine, add `target/release/` to `PATH`.
