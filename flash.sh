#!/usr/bin/env bash

powershell.exe -Command "espflash --monitor COM5 ./target/xtensa-esp32s2-espidf/debug/main"
