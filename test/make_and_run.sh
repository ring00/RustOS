realpath=$(dirname "$0")
cd $realpath
cd ../kernel
make run smp=1
