realpath=$(dirname "$0")
cd $realpath
cd ../kernel
make clean
make build smp=1
