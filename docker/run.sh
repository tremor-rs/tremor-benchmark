TREMOR_PATH="$TREMOR_PATH:$PWD/tremor-cli/tests/lib" tremor test bench tremor-cli/tests/bench -o "${1}.json" > tremor.log
cat "${1}.json"