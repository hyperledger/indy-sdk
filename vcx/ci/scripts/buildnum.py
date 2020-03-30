#!/usr/bin/env python
import time
import os

TIME_OFF_SET = 1515500000


def main():
    try:
        if os.path.exists(os.environ['CI_PROJECT_DIR'] + '/cache/build_ts'):
            with open(os.environ['CI_PROJECT_DIR'] + '/cache/build_ts', 'r') as f:
                build_num = f.read()
        else:
            build_num = str(int(time.time() - TIME_OFF_SET))
            with open(os.environ['CI_PROJECT_DIR'] + '/cache/build_ts', 'w') as f:
                f.write(build_num)
    except IOError:
        pass
    print(build_num)
    return build_num

if __name__ == '__main__':
    main()
