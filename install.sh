#!/bin/sh
set -e

# getting linux distribution name
lsb_dist=$(cat /etc/*-release | grep -o '^ID=[^,]*' | sed 's/ID=//g' | sed 's/ID=\"//g' | sed 's/\"//g')

case "$lsb_dist" in

    ubuntu|debian)
        apt-get update
        apt-get install -y libnl-3-200 libnl-3-dev
    ;;

    fedora|centos|oraclelinux)
        yum install -y wget libnl3.i686 libnl3-devel.x86_64
    ;;

esac

# putting TreeScale on your system
wget https://console.treescale.com/install/treescale
mv treescale /usr/bin/treescale
chmod +x /usr/bin/treescale
mkdir -p /etc/treescale
mkdir -p /var/log/treescale