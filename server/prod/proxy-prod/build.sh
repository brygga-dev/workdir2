#!/bin/bash

# Some gymnastics for building atm
# I prefer to build and copy over executable
# to lessen download on rustup update
# and option to use it directly

# We'll get permission errors when building below vagrant

# There is a script in `proxy` folder that copies files over
# to /vagrant/src-proxy then calls this script

cp -R /vagrant/src-proxy/* /home/vagrant/src-proxy/
pushd /home/vagrant/src-proxy
cargo build --release "$@"

popd
cp /home/vagrant/src-proxy/target/release/proxy /vagrant/prod/proxy-prod/