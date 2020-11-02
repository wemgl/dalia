# Dalia

A small commandline utility which creates shell aliases for changing directories quickly based
on a set of configured paths.

## Configuration
Dalia requires a configuration file in order to run properly. The file should contain a list of
paths, and any optional custom names, to create aliases for. All paths provided to `dalia` must be
absolute paths, anything else is invalid.

Aliases can have a custom name assigned to them by surrounding them with square brackets (`[` & `]`) and
including them at the beginning of the line. If dalia doesn't find a custom name for a particular directory,
then the alias will be the lowercase basename of the path (e.g. `/some/absolute/path` yields an alias named `path`).

#### Sample Configuration File
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
Add the following line to your shell configuration file:
```
$ eval "$(dalia aliases)"
```
This line will generate and output an alias command for configured directory in the current terminal session.
It's a good idea to include it in whichever configuration file your shell runs at the start of each session so
that the aliases are always available.

## Customization
Dalia expects to find its configuration, in a file named `config`, in the directory `~/.dalia`, but
that location can be changed by setting the `DALIA_CONFIG_PATH` environment variable to somewhere
else and putting the `config` file in there instead.
