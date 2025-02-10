#!/bin/bash

cmake --build build --config Debug --target gamedialog_cmd

./build/gamedialog_cmd cmd/data/vara.txt cmd/data/timeline1.txt cmd/data/timeline2.txt