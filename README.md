# ntm

## About this

ntm is an experimental backup tool. This tool aims to be a cross platform implementation of differential backup tool.

## Available commands

* init ... initialize a new destination
* backup execute (alias: backup e) ... execute backup
* backup remove (alias: backup r) ... mark backups matching a pattern as removed
* get ... get files from a backup
* clean ... remove marked backups and execute garbage collection

All commands accept `-d, --destination <PATH>` to specify the destination. The current directory is used by default.

## How to build

Navigate to the source directory and then run below.

```
$ cargo build
```
