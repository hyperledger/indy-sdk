#!/bin/python3
# Modifies the Cargo.toml file's version number
import sys

# opens Cargo.toml, reads the current major and minor 
# version, then adds the build number and commit
# hash, then writes this to the file
def main(filename, full_symver):
    try:
        f = open(filename, 'r') 
        output = ""
        for line in f.readlines():
            if 'version =' in line or 'version=' in line:
                output = output + 'version = \"%s\"\n' % full_symver
            else:
                output = output + line
        f.close()

        with open(filename, 'w') as f:
            f.write(output)

    except IOError:
        print("Error: Cannot find Cargo.toml file") 


# counts the number of periods
# in the string
def number_of_periods(s):
    count = 0
    for c in s:
        if c == '.':
            count = count + 1
    return count


# removes anything after and including 
# the second period in a string
def truncate(s):
    if '.' in s:
        return s[::-1][s.index('.')+1:][::-1]
    else:
        return s


# truncates the version to just major and minor
# then adds on the build number and hash
def change_version(v, b, h):
    while number_of_periods(v) > 1:
        v = truncate(v)
    s =  str(v) + "." + str(b) 
    if len(h) > 1:
        s = s + "+" + str(h)
    return s

# parses out the vesion number from a given 
# line (pulled from the Cargo.toml file)
def parse_version(s):
    index = s.index('=')
    if index > 0 and index < len(s):
        return s[index+1:].strip(' ').strip('\n').strip('\"')

    else:
        return s


if __name__ == "__main__":
    if len(sys.argv) == 4:
        filename = sys.argv[1]
        print("filename: %s" % filename)
        build = sys.argv[2]
        main(filename, build)
    elif len(sys.argv) == 2 and sys.argv[1] == '-t':
        test()
    else:
        print("USAGE: python3 change_version.py PATH BUILD_NUM HASH_NUM")
        print("PATH = path to Cargo.toml file")
        print("BUILD_NUM = build number for version")
