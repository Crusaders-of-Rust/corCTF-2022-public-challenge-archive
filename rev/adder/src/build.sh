#!/bin/sh

python3 translator.py
gcc src/main.c src/program.c -o challenge
