#!/bin/bash

PKCS12FILE=`cat .secret/pkcs12file_location`

sh ./noweb-build
cargo xwin build --target x86_64-pc-windows-msvc --release
osslsigncode sign -pkcs12 $PKCS12FILE -readpass .secret/pkcs12pass \
    -n "Home Delivery" -i https://github.com/rjhansen/home_delivery \
    -in target/x86_64-pc-windows-msvc/release/home_delivery.exe \
    -out home_delivery.exe -t http://timestamp.sectigo.com \
    -h sha256
rm -rf HomeDelivery-1.1-win64
mkdir -p HomeDelivery-1.1-win64/source
cp -r LICENSE.txt Cargo.toml config.yaml src HomeDelivery-1.1-win64/source
cp LICENSE.txt docs/*.pdf home_delivery.exe config.yaml HomeDelivery-1.1-win64
zip -r HomeDelivery-1.1-win64.zip HomeDelivery-1.1-win64
rm -rf *.exe target/x86_64-pc-windows-msvc/release/* HomeDelivery-1.1-win64
