#!/bin/bash

ci/build_xz.sh
scripts/release-deb.sh /opt/kime-out
