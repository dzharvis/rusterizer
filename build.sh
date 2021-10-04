trunk build --release
cp african_head.obj ./dist/
cp nm.tga ./dist/
cp textr23.tga ./dist/
(cd dist; python -m http.server 8081)