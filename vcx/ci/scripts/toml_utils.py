#!/usr/bin/env python3

# Reads the Cargo.toml file and extracs the major.minor version of the package

import sys
import os
import re

SO_FILE = 'libvcx.so'

def valid_line(line):
    return ('version =' in line or 'version=' in line) and ('uuid' not in line and 'rusqlite' not in line and 'indy-sys' not in line and 'rust' not in line)


# update the so file with the major minor build
def update_so(src_dir, version):
    dest  = SO_FILE + "." + version
    if os.path.isfile(dest) is False:
        # TODO: make this a python call
        cmd = "cp %s%s %s%s" % ( src_dir, SO_FILE, src_dir, dest)
        os.system(cmd)
        print('created file: %s' % dest)



# helper function -- unused now?
def _truncate(s):
    if '.' in s:
        return s[::-1][s.index('.')+1:][::-1]
    else:
        return s


# strips out the vesion number
def _strip_version(s):
    def validate_version(v):
        if len(v) < 1:
            print("error, length of version too short")
            return False
        else:
            try:
                int(v)
                return True
            except (ValueError, TypeError):
                print('error, value or type error on version %s' % v)
                return False

    if '=' in s:
        index = s.index('=')
        s = s[index+1:].strip(' ').strip('\n').strip('\"')
        ci = s.index('.')
        major = s[:ci]
        s = s[ci+1:]

        ci2 = s.index('.') if '.' in s else len(s)
        minor = s[:ci2]
        if not validate_version(major):
            print('Major Version Format in file incorrect %s' % major)
            sys.exit(1)
        if not validate_version(minor):
            print('Minor Version Format in file incorrect %s' % minor)
            print('minor: ' + minor)
            sys.exit(1)
        print('major:\t\t' + major)
        print('minor:\t\t' + minor)
        return (major, minor)

def get_version_from_file(filename):
    version = "0"
    try:
        f = open(filename, 'r')
        for line in f.readlines():
            if valid_line(line):
                version = line[line.index('=')+1:]
                version = version.strip('\n \'\"')
        f.close()
    except IOError:
        print('Error: Cannot find %s' % filename)
    return version

# extract the version from a toml file
def extract_version_from_file(filename):
    try:
        f = open(filename, 'r')
        for line in f.readlines():
            if valid_line(line):
                (major, minor) = _strip_version(line)
        f.close()
        return (major, minor)
    except IOError:
        print('Error: Cannot find %s' % filename)
        sys.exit(1)


def extract_revision(filename):
    revision = 0
    try:
        f = open(filename, 'r')
        for line in f.readlines():
            if 'revision =' in line or 'revision=' in line:
                revision = line[line.index('=')+1:]
                revision = revision.strip('\n').strip('\"').strip('\'').strip(' ')
                revision = revision[::-1]
                revision = revision.strip('\n').strip('\"').strip('\'').strip(' ')
                revision = revision[::-1]
        f.close()
        return revision
    except IOError:
        print('Error: cannot find %s' % filename)
        print('Exiting....')
        sys.exit(1)

# update the revision in a toml file
def update_revision(filename, revision):
    found = False
    try:
        o = ""
        f = open(filename, 'r')
        for line in f.readlines():
            if 'revision = ' in line or 'revision=' in line:
                found = True
                o = o + 'revision = \"%s\"\n' % revision
            else:
                o = o + line
        f.close()
        if found:
            with open(filename, 'w') as f:
                f.write(o)
            return 0
        else:
            print("error, revision entry not found in Cargo.toml")
            return 1
    except IOerror:
        print("Error: Cannot find %s, error reading/writing" % filename)

# update in toml file
def update_major_minor_build_to_toml(filename, version):
    try:
        o = ""
        f = open(filename, 'r')
        for line in f.readlines():
            if valid_line(line):
                o = o + 'version = \"%s\"\n' % (version)
            else:
                o = o + line

        f.close()
        with open(filename, 'w') as f:
            f.write(o)
    except IOError:
        print("Error: Cannot find Cargo.toml file, error reading/writing")

if __name__  == "__main__":
    if len(sys.argv) < 2:
        print("USAGE: %s path/to/Cargo.toml" % __file__)
        sys.exit(1)
    print(get_version_from_file(sys.argv[1]))
