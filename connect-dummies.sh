for i in `seq 1 5`; do
    ./catch_client/target/release/catch_client --dummy &
done
