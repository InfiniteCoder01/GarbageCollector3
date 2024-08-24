#!/usr/bin/env python3
import sys

with open(sys.argv[1], "rb") as image:
    print(image.read())
