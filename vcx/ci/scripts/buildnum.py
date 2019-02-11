#!/usr/bin/env python
import json
import time


TIME_OFF_SET = 1515500000


def main():
    build_num = (int(time.time() - TIME_OFF_SET))
    print(build_num)
    return build_num

if __name__ == '__main__':
    main()

