#!/bin/bash

ci/build_zst.sh
scripts/release-deb.sh /opt/kime-out
