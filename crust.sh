#!/bin/bash

echo "start"
# ./target/debug/crust -b exec pwd --addr-to test_user@10.88.0.2 --password-to 1234 &
./target/debug/crust -b exec pwd &
echo "PID: $!"
echo "after"
echo "$a"
