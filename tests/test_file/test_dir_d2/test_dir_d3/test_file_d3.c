#include <stdio.h>
 
int main()
{
    printf("Hello, World! \nNo\nThis is a TEST file.");
 
    return 0;
}


//[profile.dev]
//opt-level = 0
//
//[profile.release]
//opt-level = 3
//codegen-units = 1
//lto = true
//strip = true
//
//
//[dependencies]
//structopt = "0.3.26"
//termcolor = "1.2.0"
//rayon = "1.6.1"
//atty = "0.2.14"
//assert_cmd = "2.0.8"
//walkdir = "2"
//
//[target.'cfg(windows)'.dependencies]
//winapi-util = "0.1.5"
//
//[target.'cfg(windows)'.dependencies.winapi]
//version = "0.3.9"
//features = ["winerror"]
