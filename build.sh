trunk build --release
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

mkdir ../docs/african_head
cp ../res/african_head/model.obj ../docs/african_head/
cp ../res/african_head/normals.tga ../docs/african_head/
cp ../res/african_head/texture.tga ../docs/african_head/

for f in `ls -p | grep -v /`; do
    cp $f ../docs/
done

(cd ../docs; python -m http.server 8081)