#!/bin/bash

PKCS12FILE=`cat .secret/pkcs12file_location`
VERSION="1.1"
APPNAME=HomeDelivery
DIRNAME=$APPNAME-$VERSION-win64
APPURL=https://github.com/rjhansen/home_delivery
EXECUTABLE=home_delivery

cargo clean
rm -f *.zip
sh ./noweb-build.sh
cargo xwin build --target x86_64-pc-windows-msvc --release
osslsigncode sign -pkcs12 $PKCS12FILE -readpass .secret/pkcs12pass \
    -n "Home Delivery" -i $APPURL \
    -in target/x86_64-pc-windows-msvc/release/$EXECUTABLE.exe \
    -out $EXECUTABLE.exe -t http://timestamp.sectigo.com \
    -h sha256
rm -rf $DIRNAME
mkdir -p $DIRNAME/source
cp -r LICENSE.txt Cargo.toml config.yaml src $DIRNAME/source
cp LICENSE.txt docs/*.pdf $EXECUTABLE.exe config.yaml $DIRNAME
zip -r $DIRNAME.zip $DIRNAME
rm -rf *.exe $DIRNAME
cargo clean
