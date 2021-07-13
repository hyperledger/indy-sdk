current_dir=$(pwd)
source ~/.profile

echo "######Building LIBVCX######"
cd ../../../libvcx
cargo build
echo $(pwd)

echo "#####Building LibIndy#####"
cd ../../libindy
cargo build
echo $(pwd)

echo "#####Building LibNullPay####"
cd ../libnullpay
cargo build
echo $(pwd)

echo "####Building VCX JAR#####"
echo $current_dir
cd $current_dir
cd ..
./gradlew build

cd $current_dir



if [[ "$OSTYPE" == "linux-gnu" ]]; then
        sudo cp ../../../libvcx/target/debug/libvcx.so ./lib/libvcx.so
        sudo cp ../../../../libindy/target/debug/libindy.so ./lib/libindy.so
        sudo cp ../../../../libnullpay/target/debug/libnullpay.so ./lib/libnullpay.so

elif [[ "$OSTYPE" == "darwin"* ]]; then
        # Mac OSX
        cp ../../../libvcx/target/debug/libvcx.dylib ./lib/libvcx.dylib
        cp ../../../../libindy/target/debug/libindy.dylib ./lib/libindy.dylib
        cp ../../../../libnullpay/target/debug/libnullpay.dylib ./lib/libnullpay.dylib
fi