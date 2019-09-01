# todo: Investigate https://github.com/ultimate-guitar/reImage

FROM dpokidov/imagemagick
# https://github.com/dooman87/imagemagick-docker/blob/master/Dockerfile

# fedora:27

RUN yum -y update
RUN dnf install -y python37

# Go with www-data user as it appears in
# wordpress container with id 33
RUN useradd -K MAIL_DIR=/dev/null -u 33 www-data

# For dev server adding to vagrant group
RUN groupadd --gid 1000 --non-unique vagrant && usermod -a -G vagrant www-data

# Run as www-data
USER www-data

ADD server.py /opt/server.py

EXPOSE 1345
# -u for unbuffered makes prints show up in logs
ENTRYPOINT [ "python3", "-u", "/opt/server.py" ]
