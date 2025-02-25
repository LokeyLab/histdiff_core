#!/usr/bin/env bash

# This test is only done on macos

./signals_formatter_bin \
	-i  "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/cellData_examples/10uM/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv" \
	-o "/Users/dterciano/Desktop/hd_standalone_preprocessed.txt" \
	-n "/Users/dterciano/Desktop/hd_standalone_integrity.txt" 

rm "/Users/dterciano/Desktop/hd_standalone_integrity.txt"
./histdiff_bin \
	-i "/Users/dterciano/Desktop/hd_standalone_preprocessed.txt" \
	-o "/Users/dterciano/Desktop/hd_standalone.csv" \
	-c "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/platemaps/dummy_pm.csv" \
	-r "sample_type" \
	-d "id" \
	-w "384_Well" \
	--verbose

cargo test --test hd_test_macos -- --nocapture
