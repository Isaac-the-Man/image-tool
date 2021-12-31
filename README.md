# image-tool
A flexible image manipulation command line tool.

## usage
Currently there are two subcommands
- **convert**
  - Convert image(s) to a certain format, works with a file or a folder.                                                      
- **resize**
  - Resize image(s), can be a file or a folder. 

### Usage for `convert`
```
USAGE:                                                                                                                               
    image-tool.exe convert [FLAGS] [OPTIONS] <INPUT> --format <format>                                                               
                                                                                                                                     
FLAGS:                                                                                                                               
    -f, --folder     Perform resize for all images in a folder                                                                       
    -h, --help       Prints help information                                                                                         
    -V, --version    Prints version information                                                                                      
    -y, --yes        Agrees to all following prompts (eg. delete original files)                                                     
                                                                                                                                     
OPTIONS:                                                                                                                             
    -F, --format <format>    Specify the output format (eg. "PNG"). Supported format are PNG, JPEG                                   
    -o, --output <output>    Specify the output file name and path                                                                   
                                                                                                                                     
ARGS:                                                                                                                                
    <INPUT>    Specify the path of the file/folder to perform the operation
```

### Usage for `resize`
```
USAGE:                                                                                                                               
    image-tool.exe resize [FLAGS] [OPTIONS] <INPUT> --dimension <dimension>                                                          
                                                                                                                                     
FLAGS:                                                                                                                               
    -f, --folder     Perform resize for all images in a folder                                                                       
    -h, --help       Prints help information                                                                                         
    -V, --version    Prints version information                                                                                      
                                                                                                                                     
OPTIONS:                                                                                                                             
    -d, --dimension <dimension>    Specify the output dimension of the file in the form "SIZExSIZE" (eg. "64x64")                    
    -o, --output <output>          Specify the output file name and path                                                             
                                                                                                                                     
ARGS:                                                                                                                                
    <INPUT>    Specify the path of the file/folder to perform the operation
```
