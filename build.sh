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
    export CGO_CFLAGS="-I$SRC/src/tree_balancer/treelvs -I$SRC/src/tree_balancer/libipvs"
    export CGO_LDFLAGS="$SRC/src/tree_balancer/treelvs/libtreelvs.a -lnl-genl-3 -lnl-3"
    go build treescale.go
    mv treescale "$SRC"
    cd "$SRC"
}
function build
{
    echo "Building TreeScale Source"
    cd "$SRC/src/tree_balancer/libipvs/"
    echo "Compiling LibLVS"
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
    cd "src/tree_balancer/libipvs/"
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