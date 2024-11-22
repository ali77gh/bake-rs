#!/bin/bash

# the -e switch terminate script on first non zero code
set -e

# this script needs root access to copy binary to /usr/bin

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

echo 'getting "tag_name" from github...'
api_url="https://api.github.com/repos/$repo/releases/latest"
tag_name=$(curl --silent $api_url | grep '"tag_name"' | sed 's/"tag_name": "//' | sed 's/",//' | sed 's/ //g')
download_url="https://github.com/$repo/releases/download/$tag_name/$release_file_name"

echo ""
echo "--- installing bake version: '$tag_name' on platform: '$platform' to path: '$install_path' ---"
echo ""

echo "downloading from: '$download_url'"

curl -sfL $download_url --output bake.tar.gz
echo "download compeleted!"

echo ""
echo "extracting bake.tar.gz"
tar -xvzf bake.tar.gz bake
echo "bake.tar.gz extracted!"

echo ""
echo "moveing bake to $install_path"
mv bake "$install_path"
echo "bake is moved to $install_path"

echo ""
echo "removing bake.tar.gz"
rm bake.tar.gz
echo "bake.tar.gz is removed!"

echo ""
echo "bake installed successfully"
echo "running: 'bake --version'"
bake --version
