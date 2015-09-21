#!/bin/bash
set -e

BUILD_CLEAN="$1"
SRC=`pwd`

if [ "$BUILD_CLEAN" == "" ]; then
    BUILD_CLEAN="build"
fi

function go_build
{
    echo "Building TreeScale executable"
    cd "$SRC/src"
    export CGO_CFLAGS="-I$SRC/src/balancer/treelvs -I$SRC/src/balancer/libipvs -I$SRC/src/balancer/libipvs/libnl1/include"
    export CGO_LDFLAGS="$SRC/src/balancer/treelvs/libtreelvs.a $SRC/src/balancer/libipvs/libnl1/build/lib/libnl.a -lm"
    go build treescale.go
    mv treescale "$SRC"
    cd "$SRC"
}
function build
{
    echo "Building TreeScale Source"
    echo "Compiling Letlink Library"
    cd "$SRC/src/balancer/libipvs/libnl1/"
    ./configure --prefix=`pwd`/build
    make
    make install
    echo "Compiling LibLVS"
    cd ./../
    make
    cp libipvs.a ./../treelvs/
    echo "Compiling TreeLVS"
    cd ./../treelvs/
    gcc -c treelvs.c -o treelvs.o -I../libipvs
    echo "Making Library for GO"
    ar -x libipvs.a
    ar rcs libtreelvs.a *.o
    echo "Cleaning up"
    rm -f libipvs.a *.o
    go_build
}

function clean
{
    cd src/balancer/libipvs/libnl1
    make clean
    rm -rf build
    cd ../
    make clean
    cd ../treelvs
    rm -f *.[ao] *~ *.orig *.rej core *.so
    cd $GOPATH
    rm -f treescale
}


function publish
{
    PUB_PATH="$2"
    if [ "$PUB_PATH" == "" ]; then
        PUB_PATH="tigran@console.treescale.com:/home/tigran/treescale-console/treescale"
    fi

    scp ./treescale "$PUB_PATH"
    scp ./src/install.sh "$PUB_PATH"
}

if [ "$BUILD_CLEAN" == "build" ]; then
build
fi

if [ "$BUILD_CLEAN" = "clean" ]; then
clean
fi

if [ "$BUILD_CLEAN" = "go" ]; then
go_build
fi

if [ "$BUILD_CLEAN" = "publish" ]; then
go_build # building then publishing
publish
fi