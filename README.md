# Dalia

A small commandline utility for creating shell aliases to change directories quickly without needing to type `cd`.

## Configuration
Dalia requires a configuration file in order to run properly. Dalia expects the configuration file to be at `$HOME/.dalia/config`
by default. The file should contain a list of absolute paths, and any optional custom names at the start of the line, to create all aliases.
Finally, all configured paths must be absolute paths—anything else is invalid.

### Custom Alias Names
Aliases can have a custom name assigned to them, just surround whatever text you want with square brackets (`[` & `]`) and
include it at the beginning of the line. If dalia doesn't find a custom name for a particular directory,
then the alias will be the lowercase basename of the absolute path (e.g. `/some/absolute/path` yields an alias named `path`).

#### Configuration File Example
Here's an example of a configuration file that `dalia` would load from `$HOME/dalia/config`:
```
[workspace]~/Documents/workspace
~/Desktop
[icloud]~/Library/Mobile\ Documents/com~apple~CloudDocs
/Users/johnappleseed/Music
[photos] /Users/johnappleseed/Pictures
```
This configuration file will create the following aliases:
```
workspace='cd ~/Documents/workspace'
desktop='cd ~/Desktop'
icloud='cd '~/Library/Mobile\ Documents/com~apple~CloudDocs'
music='cd /Users/johnappleseed/Music'
photos='cd /Users/johnappleseed/Pictures'
```

## Installation
Install like any other Rust crate with:
```
$ cargo install dalia
```
Then, add the following line to your shell configuration file:
```
$ eval "$(/path/to/cmd/dalia aliases)"
```
This line will generate and output an alias command for configured directory in the current terminal session.
It's a good idea to include it in whichever configuration file your shell runs at the start of each session so
that the aliases are always available.

## Customization
Dalia expects to find its configuration, in a file named `config`, in the directory `~/.dalia`, but
that location can be changed by setting the `DALIA_CONFIG_PATH` environment variable to somewhere
else and putting the `config` file in there instead.
