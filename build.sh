trunk build --release
cp african_head.obj ./dist/
cp nm.tga ./dist/
cp textr23.tga ./dist/
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
  
<link rel="preload" href="$wasm_file" as="fetch" type="application/wasm" crossorigin="">
<link rel="modulepreload" href="$js_file"></head>
<body><script type="module">import init from './$js_file';init('./$wasm_file');</script></body></html>
EOM

rm -rf ../docs/*

for f in ./*; do
    cp $f ../docs/
done

(python -m http.server 8081)