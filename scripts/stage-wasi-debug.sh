#!/bin/bash

echo $1 $2

if [ $# -ne 2 ]
then
    echo "Bad number of args"
else
    cp -f $1/target/wasm32-wasi/debug/rproxy.wasm $2/bitcode/wasm/rproxy/rproxy.wasm
    cp -f $1/target/wasm32-wasi/debug/image.wasm $2/bitcode/wasm/image/image.wasm
    cp -f $1/target/wasm32-wasi/debug/real_img.wasm $2/bitcode/wasm/image/real_img.wasm
    cp -f $1/target/wasm32-wasi/debug/search.wasm $2/bitcode/wasm/search/search.wasm
    cp -f $1/target/wasm32-wasi/debug/everything.wasm $2/bitcode/wasm/everything/everything.wasm
    cp -f $1/samples/library/library.wasm $2/bitcode/wasm/library/library.wasm
    cp -f $1/samples/proxy/proxy.wasm $2/bitcode/wasm/proxy/proxy.wasm
fi

