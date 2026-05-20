#!/bin/bash
mkdir -p dist/assets/logo/png dist/assets/icon dist/assets/css dist/assets/js
cp src/web_ui/index.html dist/
cp src/web_ui/sw.js dist/
cp src/web_ui/manifest.json dist/assets/
cp src/web_ui/css/* dist/assets/css/
cp src/web_ui/js/* dist/assets/js/
cp assets/logo/* dist/assets/logo/
cp assets/logo/png/* dist/assets/logo/png/
cp assets/icon/* dist/assets/icon/
