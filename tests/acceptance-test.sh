#!/usr/bin/env bash

DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
DIR="$DIR/..";
cd $DIR

cargo build -r

for source_file in $DIR/tests/source/*; do
    filename=$(basename $source_file)
    echo "Generating tiles for $filename"
    output_dir="$DIR/tests/tmp/$filename"
    mkdir -p $output_dir
    ./target/release/image2slippytiles --resumable --thumbnail --colour '#DDDDDDFF' --format png --zoom 0 --json --output $output_dir "$source_file" | jq 'del(.peak_memory) | del(.duration)' > "./tests/tmp/$filename.out"
done

echo "Comparing output to expected results";
find "./tests/tmp" -type f -exec md5sum {} \; | sort &> "./tests/tmp/acceptance-test.results"
find "./tests/tmp" -name "*.png" -exec file {} \; | sort &>> "./tests/tmp/acceptance-test.results"
diff=$(diff ./tests/tmp/acceptance-test.results ./tests/expected/acceptance-test.results)

if [ $? == 1 ]; then
    echo -e "Acceptance testing failed. Unexpected output:
    $diff"
    exit 1;
else
    echo "Acceptance tests passed."
    rm -Rf ./tests/tmp
fi