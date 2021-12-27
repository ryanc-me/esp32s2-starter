#!/usr/bin/env bash

powershell.exe -Command "espflash --speed 460800 COM5 ./target/xtensa-esp32s2-espidf/debug/main; espmonitor COM5"
