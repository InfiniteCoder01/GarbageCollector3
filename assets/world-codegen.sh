#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
ldtk-codegen "$SCRIPT_DIR/../src/world.ldtk" \
		--vector 'speedy2d::dimen::Vector2<T>' \
		--color 'speedy2d::color::Color' \
