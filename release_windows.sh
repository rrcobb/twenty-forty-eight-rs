#!/bin/sh
# show the commands that are being run
set -x

# make a windows release dir
rm -rf 2048-windows
mkdir 2048-windows

# copy *.dll files 
cp ./windows_resources/* ./2048-windows/

# copy .exe
cp ./target/x86_64-pc-windows-gnu/release/twenty-forty-eight.exe ./2048-windows/

# zip them up
zip -r 2048-windows.zip ./2048-windows
