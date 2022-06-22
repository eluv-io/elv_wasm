#!/bin/bash

echo $1 $2

if [ $# -ne 2 ]
then
    echo "Bad number of args"
else
    wasm-gc $1/target/wasm32-unknown-unknown/release/rproxy.wasm $2/bitcode/wasm/rproxy/rproxy.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/image.wasm $2/bitcode/wasm/image/image.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/real_img.wasm $2/bitcode/wasm/image/real_img.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/search.wasm $2/bitcode/wasm/search/search.wasm
    wasm-gc $1/target/wasm32-unknown-unknown/release/everything.wasm $2/bitcode/wasm/everything/everything.wasm
    wasm-gc $1/samples/library/library.wasm $2/bitcode/wasm/library/library.wasm
    wasm-gc $1/samples/proxy/proxy.wasm $2/bitcode/wasm/proxy/proxy.wasm
fi

