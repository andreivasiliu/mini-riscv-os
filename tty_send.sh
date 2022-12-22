#!/bin/bash

set -e

file_name=$1

base_name=$(basename $file_name)
file_size=$(stat -c %s $file_name)

echo -en "paste $base_name $file_size\r" > /dev/ttyUSB0
sleep 0.1
cat "$file_name" > /dev/ttyUSB0
