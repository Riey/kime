#!/bin/bash

scripts/build.sh -ar
tar cvf - -C ./build/out . | xz -9 -T0 -c - > /opt/kime-out/kime.tar.xz

