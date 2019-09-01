sudo yum update -y
sudo yum install docker -y
sudo service docker start
sudo usermod -a -G docker ec2-user
sudo curl -L "https://github.com/docker/compose/releases/download/1.24.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
# Might need logout, log int
sudo yum install git -y
git clone https://brygga-dev@github.com/brygga-dev/brygga.git
cd brygga/vagrant/prod
source prod.sh
