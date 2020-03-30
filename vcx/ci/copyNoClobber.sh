#!/bin/bash

if [ $# -ne "1" ]; then
    echo USAGE: `basename "$0"` INDY_DIR
    exit 1
elif [ ! -e $INDY ]; then
    echo "Incorrect Indy Location"
fi

INDY=$1

ls -al
ls -alr vcx/*
ls -alr vcx/libvcx/*


cp -rn $INDY/vcx/* vcx
mkdir -p wrappers/rust
cp -rn $INDY/wrappers/rust/* wrappers/rust
ls -al
ls -alr vcx/*
ls -alr vcx/libvcx/*
