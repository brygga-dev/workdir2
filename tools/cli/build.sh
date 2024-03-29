#!/bin/bash
RUST_BACKTRACE=1 cargo build
CLI_X="/usr/local/bin/wop"
if [ -e "$CLI_X" ]; then
        sudo rm "$CLI_X"
fi
# sudo ln ../target/debug/project-api "$CLI_X"
sudo bash -c "echo '#!/bin/bash' >> $CLI_X"
sudo bash -c "echo 'RUST_BACKTRACE=1 $(pwd)/../target/debug/project-api \"\$@\"' >> $CLI_X"
sudo chmod +x "$CLI_X"
source /home/vagrant/wop.bash
