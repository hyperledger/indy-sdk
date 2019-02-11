#!/bin/sh
#1) Install Rust and rustup (https://www.rust-lang.org/install.html).
#To get into the if statement below execute the following command...
# mv /Users/norm/.cargo/bin/rustup /Users/norm/.cargo/bin/rustup.bak
RUSTUP_VERSION=`rustup --version`
DEFAULT_RUST_VERSION=$1
if [ "$?" != "0" ]; then
    if [ -f $HOME/.cargo/bin/rustup ]; then
        echo "You need to add $HOME/.cargo/bin to your PATH environment variable or simply restart your terminal"
        exit 1
    else
        if [ -f /usr/local/bin/rustc ]; then
            sudo mv /usr/local/bin/rustc /usr/local/bin/rustc.bak
            sudo mv /usr/local/bin/rustdoc /usr/local/bin/rustdoc.bak
            sudo mv /usr/local/bin/rust-lldb /usr/local/bin/rust-lldb.bak
            sudo mv /usr/local/bin/rust-gdb /usr/local/bin/rust-gdb.bak
        fi
        if [ -d /usr/local/lib/rustlib ]; then
            sudo mv /usr/local/lib/rustlib /usr/local/lib/rustlib.bak
            sudo mkdir /usr/local/lib/rustlib.bak/libs
            sudo mv /usr/local/lib/librustc* /usr/local/lib/rustlib.bak/libs
        fi
        curl https://sh.rustup.rs -sSf | sh
        source $HOME/.cargo/env
        rustup component add rust-src
        rustup component add rust-docs
        rustup update
        RUSTUP_VERSION=`rustup --version`
        if [ -f /usr/local/bin/rustc.bak ]; then
            sudo mv /usr/local/bin/rustc.bak /usr/local/bin/rustc
            sudo mv /usr/local/bin/rustdoc.bak /usr/local/bin/rustdoc
            sudo mv /usr/local/bin/rust-lldb.bak /usr/local/bin/rust-lldb
            sudo mv /usr/local/bin/rust-gdb.bak /usr/local/bin/rust-gdb
        fi
        if [ -d /usr/local/lib/rustlib.bak ]; then
            sudo mv /usr/local/lib/rustlib.bak /usr/local/lib/rustlib
            sudo mv /usr/local/lib/rustlib/libs/* /usr/local/lib
            sudo rm -rf /usr/local/lib/rustlib/libs
        fi
    fi
fi
# Steps to uninstall rustup to test that the step 1) works again
# rustup self uninstall

if [[ $RUSTUP_VERSION =~ ^'rustup ' ]]; then
    rustup update
    rustup default ${DEFAULT_RUST_VERSION}
    rustup component add rls-preview rust-analysis rust-src
    echo "Using rustc version $(rustc --version)"
    rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios

    RUST_TARGETS=$(rustc --print target-list|grep -i ios)
    if [ "$RUST_TARGETS" = "" ]; then
        sudo xcodebuild -license
        # DON'T do this
        #xcode-select --install # Install Command Line Tools if you haven't already.
        #sudo xcode-select --switch /Library/Developer/CommandLineTools
        # INSTEAD do this
        sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
        echo "If you have successfully accepted the xcode build license then just re-run this script."
        echo "If you were not able to successfully accept the xcode build license then run this command in a terminal 'sudo xcodebuild -license' until it is successful before you attempt to re-run this script"
        exit 1
    fi

    cargo install --force cargo-lipo
    cargo install --force cargo-xcode

    BREW_VERSION=`brew --version`
    if ! [[ $BREW_VERSION =~ ^'Homebrew ' ]]; then
        /usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
        brew doctor
        brew update
    fi

    #2) Install required native libraries and utilities (libsodium is added with URL to homebrew since version<1.0.15 is required)
    brew install pkg-config
    brew install https://raw.githubusercontent.com/Homebrew/homebrew-core/65effd2b617bade68a8a2c5b39e1c3089cc0e945/Formula/libsodium.rb
    brew install automake
    brew install autoconf
    brew install cmake
    brew install openssl
    brew install zmq
    brew install wget
    brew install truncate
    brew install libzip
    brew install python3
fi
