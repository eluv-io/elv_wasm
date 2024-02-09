#!/bin/bash

echo $1 $2

if [ $# -ne 2 ]; then
    echo "Bad number of args"
else
    cp $1/target/wasm32-unknown-unknown/release/rproxy.wasm $2/bitcode/wasm/rproxy/rproxy.wasm
    cp $1/target/wasm32-unknown-unknown/release/real_img.wasm $2/bitcode/wasm/image/real_img.wasm
    cp $1/target/wasm32-unknown-unknown/release/search.wasm $2/bitcode/wasm/search/search.wasm
    cp $1/target/wasm32-unknown-unknown/release/objtar.wasm $2/bitcode/wasm/objtar/objtar.wasm
    cp $1/target/wasm32-unknown-unknown/release/lro.wasm $2/bitcode/wasm/lro/lro.wasm
    cp $1/target/wasm32-unknown-unknown/release/external.wasm $2/bitcode/wasm/external/external.wasm

    # cp -u -f $1/target/wasm32-unknown-unknown/release/test_wapc.wasm $2/exeng/tests

    # cp $1/samples/proxy/proxy.wasm $2/bitcode/wasm/proxy/proxy.wasm
fi
