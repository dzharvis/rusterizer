#!/bin/bash

cat > index.html <<- EOM
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Rusterizer</title>
  </head>
</html>
EOM
RUSTFLAGS="-C target-feature=+simd128 -C opt-level=3" ./trunk build --release
rm index.html
cd dist

pattern="index-*_bg.wasm"
files=( $pattern )
wasm_file="${files[0]}"

pattern="index-*.js"
files=( $pattern )
js_file="${files[0]}"

cat > index.html <<- EOM
<!DOCTYPE html><html><head>
    <meta charset="utf-8">
    <title>Rusterizer</title>
<link rel="stylesheet" href="index.css">
<link rel="preload" href="$wasm_file" as="fetch" type="application/wasm" crossorigin="">
<link rel="modulepreload" href="$js_file"></head>
<body><script type="module">import init from './$js_file';init('./$wasm_file');</script></body></html>
EOM

rm -rf ../docs/*

mkdir ../docs/african_head
mkdir ../docs/diablo
cp ../res/african_head/model.obj ../docs/african_head/
cp ../res/african_head/normals.tga ../docs/african_head/
cp ../res/african_head/texture.tga ../docs/african_head/
cp ../res/diablo/model.obj ../docs/diablo/
cp ../res/diablo/normals.tga ../docs/diablo/
cp ../res/diablo/texture.tga ../docs/diablo/

cp ../static/index.css ../docs/

for f in `ls -p | grep -v /`; do
    cp $f ../docs/
done

(cd ../docs; python -m http.server 8081)