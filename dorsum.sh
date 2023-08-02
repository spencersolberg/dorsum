#!/bin/bash

# Ensure the script is run as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi

useradd -M -r dorsum

hostname "dorsum"
OLD_HOSTNAME=$(cat /etc/hostname)
echo "dorsum" > /etc/hostname
sed -i "s/$OLD_HOSTNAME/dorsum/g" /etc/hosts

mkdir -p /etc/dorsum/certificates
mkdir -p /opt/dorsum/bin

chown dorsum:dorsum /etc/dorsum
chown dorsum:dorsum /opt/dorsum

apt update
apt full-upgrade

# prompt user if they would like to install tailscale
echo "Would you like to install Tailscale? (y/n)"
read tailscale

# Enable case-insensitive string comparison
shopt -s nocasematch

if [ "$tailscale" == "y" ]; then
  # tailscale
  curl -fsSL https://tailscale.com/install.sh | sh
  sudo tailscale up
fi

# Disable case-insensitive string comparison
shopt -u nocasematch

# hnsd
apt install -y autotools-dev libtool libunbound-dev git make build-essential
cd /tmp
git clone https://github.com/handshake-org/hnsd
cd hnsd
./autogen.sh && ./configure && make
mv /tmp/hnsd/hnsd /opt/dorsum/bin/hnsd
rm -rf /tmp/hnsd
setcap 'cap_net_bind_service=+ep' /opt/dorsum/bin/hnsd

sudo tee /etc/systemd/system/dorsum-hnsd.service > /dev/null << EOL
[Unit]
Description=Dorsum HNSD Service
After=network.target

[Service]
Type=simple
User=dorsum
ExecStart=/opt/dorsum/bin/hnsd -p 4 -r 0.0.0.0:53
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOL

systemctl enable --now dorsum-hnsd

# go
export GOROOT=/tmp/go
curl https://raw.githubusercontent.com/canha/golang-tools-install-script/master/goinstall.sh | bash

# letsdane
cd /tmp
git clone https://github.com/buffrr/letsdane
cd letsdane/cmd/letsdane
/tmp/go/bin/go build -tags unbound
mv /tmp/letsdane/cmd/letsdane/letsdane /opt/dorsum/bin/letsdane
rm -rf /tmp/letsdane
mkdir /etc/dorsum/letsdane/
chown -R dorsum /etc/dorsum/letsdane/
chmod 700 /etc/dorsum/letsdane/
/opt/dorsum/bin/letsdane -o /etc/dorsum/certificates/letsdane.crt


sudo tee /etc/systemd/system/dorsum-letsdane.service > /dev/null << EOL
[Unit]
Description=Dorsum Let's DANE Service
After=network.target

[Service]
Type=simple
User=dorsum
ExecStart=/opt/dorsum/bin/letsdane -r 127.0.0.1:53 -skip-icann -skip-dnssec -conf /etc/dorsum/letsdane
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOL

systemctl enable --now dorsum-letsdane

# mkcert
cd /tmp
apt install -y libnss3-tools
git clone https://github.com/FiloSottile/mkcert
cd mkcert
/tmp/go/bin/go build -ldflags "-X main.Version=$(git describe --tags)"
mv /tmp/mkcert/mkcert /opt/dorsum/bin/mkcert
rm -rf /tmp/mkcert

mkdir /etc/dorsum/mkcert
chown -R dorsum /etc/dorsum/mkcert
export CAROOT=/etc/dorsum/mkcert

if [ "$tailscale" == "y" ]; then
  cd /etc/dorsum/certificates
  /opt/dorsum/bin/mkcert -cert-file dorsum.crt -key-file dorsum.key dorsum.local $(tailscale ip -4)
else
  cd /etc/dorsum/certificates
  /opt/dorsum/bin/mkcert -cert-file dorsum.crt -key-file dorsum.key dorsum.local 
fi

mv /etc/dorsum/mkcert/rootCA.pem /etc/dorsum/certificates/dorsum-root.crt

# cat /etc/dorsum/letsdane/letsdane.crt /etc/dorsum/mkcert/rootCA.pem > /etc/dorsum/dorsum.crt

sudo chmod -R o+r /etc/dorsum/certificates

# dnsproxy
cd /tmp
git clone https://github.com/AdguardTeam/dnsproxy
cd dnsproxy
/tmp/go/bin/go build -mod=vendor
mv /tmp/dnsproxy/dnsproxy /opt/dorsum/bin/dnsproxy
rm -rf /tmp/dnsproxy
setcap 'cap_net_bind_service=+ep' /opt/dorsum/bin/dnsproxy

sudo tee /etc/systemd/system/dorsum-dnsproxy.service > /dev/null << EOL
[Unit]
Description=Dorsum DNS Proxy
After=network.target

[Service]
Type=simple
User=dorsum
ExecStart=/opt/dorsum/bin/dnsproxy -l 0.0.0.0 --tls-port=853 --https-port=444 --tls-crt=/etc/dorsum/certificates/dorsum.crt --tls-key=/etc/dorsum/certificates/dorsum.key -u 127.0.0.1:53 -p 0
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOL

systemctl enable --now dorsum-dnsproxy

# remove go
rm -rf /tmp/go

# caddy
apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
apt update
apt install -y caddy

# deno
apt install -y unzip
curl -s https://gist.githubusercontent.com/LukeChannings/09d53f5c364391042186518c8598b85e/raw/ac8cd8c675b985edd4b3e16df63ffef14d1f0e24/deno_install.sh | sh
# /root/.deno/bin/deno