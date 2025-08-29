#!/bin/bash
set -a 

source .env 


cargo check --lib --features "local-lib,mock-provab,debug_display" && cargo check --bin estate-fe --features "local-bin,mock-provab,debug_display"

set +a