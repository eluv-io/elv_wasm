#!/bin/bash

if ! command -v wasm-gc &> /dev/null
then
    echo "wasm-gc could not be found, please install via cargo install wasm-gc"
    exit
fi

echo $1 $2

if [ $# -ne 2 ]
then
    echo "Bad number of args"
else
    wasm-gc $1/target/wasm32-unknown-unknown/release/rproxy.wasm $2/bitcode/wasm/rproxy/rproxy.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/real_img.wasm $2/bitcode/wasm/image/real_img.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/search.wasm $2/bitcode/wasm/search/search.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/everything.wasm $2/bitcode/wasm/everything/everything.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/objtar.wasm $2/bitcode/wasm/objtar/objtar.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/lro.wasm $2/bitcode/wasm/lro/lro.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/external.wasm $2/bitcode/wasm/external/external.wasm

    cp -u -f $1/target/wasm32-unknown-unknown/release/test_wapc.wasm $2/exeng/tests

    wasm-gc $1/samples/proxy/proxy.wasm $2/bitcode/wasm/proxy/proxy.wasm
fi

