#!/usr/bin/env bash

# Stop subsequent execution when encountering any errors
set -e

TARGET=${1}
RELEASE_MODE=${2}

if [ ! ${TARGET} ]; then
    : ${TARGET:=aarch64-apple-ios}
fi

if [ "${TARGET}" = "--release" ]; then
    TARGET="aarch64-apple-ios"
    : ${RELEASE_MODE:=--release}
fi

cargo build --target ${TARGET} ${RELEASE_MODE}

LIB_FOLDER=
case ${RELEASE_MODE} in
    "--release") : ${LIB_FOLDER:=release} ;;
    *) : ${LIB_FOLDER:=debug} ;;
esac

if [ ! -d "xcode/${LIB_FOLDER}/" ]; then
  mkdir -p "xcode/${LIB_FOLDER}"
fi

cp "../target/${TARGET}/${LIB_FOLDER}/libinvapp.a" xcode/${LIB_FOLDER}
