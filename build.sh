#! /bin/bash

pushd crepe
    pip install -e .
popd

pushd fingering
    cargo build --release
popd

pushd notes
    make
popd