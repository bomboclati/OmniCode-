#!/bin/bash
mkdir -p dist/app/assets/logo/png dist/app/assets/icon dist/app/assets/css dist/app/assets/js
mkdir -p dist/assets/icon
cp landing-page/index.html dist/
cp src/web_ui/index.html dist/app/
cp src/web_ui/sw.js dist/app/
cp src/web_ui/manifest.json dist/app/assets/
cp src/web_ui/css/* dist/app/assets/css/
cp src/web_ui/js/* dist/app/assets/js/
cp assets/logo/* dist/app/assets/logo/
cp assets/logo/png/* dist/app/assets/logo/png/
cp assets/icon/* dist/app/assets/icon/
cp assets/icon/* dist/assets/icon/
cp assets/omnicode.apk dist/omnicode.apk 2>/dev/null || true
