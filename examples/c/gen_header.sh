#!/bin/sh


# NOTE we call cbindgen outside of build.rs because cbindgen expand seems to 
#      suck when called from a build.rs file
cbindgen --config cbindgen.toml --lang c --clean . | clang-format > target/binding.h
