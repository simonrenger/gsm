# gsm
Git Submodule utility for easy add, update and remove of submodules

## usage

```
USAGE:
    gsm [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -l, --list       list submodule
    -V, --version    Prints version information

OPTIONS:
    -a, --add <submdoule>       Adds a submodule
    -n, --name <name>           names a submodule
    -r, --remove <submdoule>    removes a submodule
    -u, --update <submdoule>    updates submodule to latests
 ```
 
 *example:*
 
 ```bash
 > ./gsm -a https://github.com/simonrenger/stainless -n third_party/stainless
 > ./gsm -u third_party/stainless
 > ./gsm -r third_party/stainless
 ```
 
