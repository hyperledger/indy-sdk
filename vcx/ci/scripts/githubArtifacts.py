#!/usr/bin/env python3
import sys
import os
import json
import requests
import argparse

def main(file_list):
    """main program body"""

    #check for required variables
    env_set = True
    try:
        # github_repo - should be 'account/repo', ie: 'evernym/mobile-sdk'
        github_repo = os.environ['GITHUB_REPO']
    except KeyError:
        print("Missing required environment variable. Please set 'GITHUB_REPO'")
        env_set = False

    try:
        # github_token - github personal access token
        github_token = os.environ['GITHUB_TOKEN']
    except KeyError:
        print("Missing required environment variable. Please set 'GITHUB_TOKEN'")
        env_set = False

    try:
        release_tag = os.environ['CI_PIPELINE_ID']
    except KeyError:
        print("Missing required environment variable. Please set 'CI_PIPELINE_ID'")
        env_set = False

    if not env_set:
        sys.exit(1)


    # release_tag - os.environ['CI_PIPELINE_ID']
    # file_list - list of files to upload

    # create release
    #github_repo = "evernym/mobile-sdk"
    github_url = "https://api.github.com/repos/" + github_repo + "/releases"
    post_data = {
        "tag_name": release_tag,
        "name": release_tag,
        "target_commitish": "master"
    }

    print("Creating release '{}' on github repo '{}'".format(release_tag, github_repo))
    try:
        response = requests.post(url=github_url, headers={"Content-Type": "application/json", "Authorization": "token " + github_token}, data=json.dumps(post_data).encode("UTF-8"))
        response.raise_for_status()
    except requests.exceptions.HTTPError:
        print("Error creating release on github. [{}] {} {} \"{}\"".format(github_url, response.status_code, response.reason, response.content))
        sys.exit(1)

    # get upload url from previous response
    try:
        assets_url = response.json()['upload_url'].split('{')[0]
        html_url = response.json()['html_url']
    except KeyError:
        print("Error getting upload_url from github response. [{}] {} {} \"{}\"".format(github_url, response.status_code, response.reason, response.content))
        sys.exit(1)

    #upload files
    for filename in file_list:
        basename = filename.split('/')[-1]
        if basename == filename:
            print("Uploading file: '{}'".format(filename))
        else:
            print("Uploading file: '{}' as '{}'".format(filename, basename))
        upload_url = assets_url + "?name=" + basename
        try:
            with open(filename, 'rb') as rawfile:
                response = requests.post(url=upload_url, headers={"Content-Type": "application/octet-stream", "Authorization": "token " + github_token}, data=rawfile.read())
            response.raise_for_status()
        except requests.exceptions.HTTPError:
            print("Error uploading file. [{}] {} {} \"{}\"".format(upload_url, response.status_code, response.reason, response.content))
        except IOError:
            errormsg = "unknown"
            print("Could not open file for reading: \"{}\" {}".format(filename, errormsg))

    print("Completed. {}".format(html_url))
## end main() ##

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('file_list', nargs="*", help='Additional artifacts to upload')
    args = parser.parse_args()
    main(file_list=args.file_list)
