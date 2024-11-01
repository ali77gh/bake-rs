#!/bin/bash

# this script needs root access

repo="ali77gh/bake-rs"
binary_name="bake"

platform=`uname`

if [ "$platform" = "Linux" ]
then
    install_path="/usr/bin/$binary_name"
    release_file_name="bake-Linux-musl-x86_64.tar.gz"
fi
if [ "$platform" = "Darwin" ]
then
    install_path="/usr/local/bin/$binary_name"
    release_file_name="bake-macOS-x86_64.tar.gz"
fi

echo "downloading..."
tag_name=$(curl --silent https://api.github.com/repos/$repo/releases/latest \
                  | grep '"tag_name"' \
                  | sed --regexp-extended 's/.*"([^"]+)".*/\1/')

curl -sfL "https://github.com/$repo/releases/download/$tag_name/$release_file_name" --output bake.tar.gz
echo "download compeleted!"

echo "extracting bake.tar.gz"
tar -xvzf bake.tar.gz bake
echo "bake.tar.gz extracted!"

echo "moveing bake to $install_path"
mv bake "$install_path"
echo "bake is moved to $install_path"

echo "removing bake.tar.gz"
rm bake.tar.gz
echo "bake.tar.gz is removed!"

echo "bake installed successfully"

echo "running: bake --version"
bake --version