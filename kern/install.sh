#!/bin/sh
sudo mount /dev/sdc1 /mnt/sdcard
echo "SUccessfully mounted SD Card ..."
cd ../user/$1/
sudo rm ./build/$1.bin
sudo env "PATH=$PATH" make
sudo cp ./build/$1.bin /mnt/sdcard
echo "Unmounting the SD Card ..."
sudo umount /mnt/sdcard
echo "All done (:"
