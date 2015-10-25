for i in `seq 1 10`; do
    ./catch_client/target/release/catch_client --dummy &
done
