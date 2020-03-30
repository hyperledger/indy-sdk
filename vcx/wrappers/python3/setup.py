from setuptools import setup, find_packages
import os

PKG_VERSION = os.environ.get('PACKAGE_VERSION') or '0.7.1'
PKG_NAME = os.environ.get('PACKAGE_NAME') or 'python3-wrapper-vcx'

setup(
    name=PKG_NAME,
    version=PKG_VERSION,
    description='Python 3 wrapper for libcxs',
    long_description='None...for now',
    author="Hyperledger Indy Contributors",
    author_email= "hyperledger-indy@lists.hyperledger.org",
    include_package_data=True,
    packages=find_packages(exclude=['demo', 'tests']),
    license='Apache-2.0',
)
