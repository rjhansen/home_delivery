#!/bin/bash

# If you're on a Mac, install gsed from Homebrew.

gnused=/usr/bin/sed
if [ -f /opt/homebrew/bin/gsed ]; then
    echo "Using true GNU sed…"
    gnused='/opt/homebrew/bin/gsed'
fi

for x in main.rs utility.rs ;
do
notangle -R$x src/code.nw > src/$x
rustfmt src/$x
done

notangle -RCargo.toml src/code.nw > Cargo.toml
notangle -Rconfig.yaml src/code.nw > config.yaml

noweave -delay src/code.nw -x > HomeDelivery.tex
md5main=`md5sum src/main.rs | cut -d " " -f 1`
md5utility=`md5sum src/utility.rs | cut -d " " -f 1`
md5cargo=`md5sum Cargo.toml | cut -d " " -f 1`
md5config=`md5sum config.yaml | cut -d " " -f 1`
$gnused -i "s/MAIN_MD5/$md5main/" HomeDelivery.tex
$gnused -i "s/UTILITY_MD5/$md5utility/" HomeDelivery.tex
$gnused -i "s/CARGO_MD5/$md5cargo/" HomeDelivery.tex
$gnused -i "s/CONFIG_MD5/$md5config/" HomeDelivery.tex
pdflatex HomeDelivery.tex
pdflatex HomeDelivery.tex
rm -f *.aux *.log *.out *.toc *.tex
rm -rf docs
mkdir docs
mv *.pdf docs

cargo build --release

# To build a Windows release on Linux or MacOS:
#
# 1. Install osslsigncode (https://github.com/mtrojnar/osslsigncode)
# 2. Get a code signing cert in PKCS12 format
# 3. Install the Rust cross compiler (https://github.com/rust-cross/cargo-xwin)
# 4. cargo xwin build --target x86_64-pc-windows-msvc --release
# 5. osslsigncode sign -pkcs12 <pkcs12-file> -pass <pkcs12-password> \
#        -n "Home Delivery" -i https://github.com/rjhansen/home_delivery \
#        -in target/x86_64-pc-windows-msvc/release/home_delivery.exe 
#        -out home_delivery.exe -t http://timestamp.sectigo.com
#        -h sha256
# 6. rm -rf HomeDelivery-1.1-win64
# 7. mkdir -p HomeDelivery-1.1-win64/source
# 8. cp -r Cargo.toml config.yaml src HomeDelivery-1.1-win64/source
# 9. cp LICENSE.txt docs/*.pdf home_delivery.exe config.yaml HomeDelivery-1.1-win64
# 10. zip -r HomeDelivery-1.1-win64.zip HomeDelivery-1.1-win64

