cd wasm-demo
wasm-pack build --release
cd pkg
npm link
cd ..
cd www
npm install
npm link wasm-demo
npm run build
cd ../..
