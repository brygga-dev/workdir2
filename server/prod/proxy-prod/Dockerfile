# todo: Incorporate changes from -dev

# Using bionic as vagrant, currently the build is done on vagrant
# then copied here
FROM ubuntu:bionic

RUN apt-get update && apt-get install -y --no-install-recommends apt-utils

RUN DEBIAN_FRONTEND=noninteractive apt-get install -y \
	pkg-config \
	libssl-dev \
	libssl1.1 \
	ca-certificates

#RUN apt-get install -y openssl
# apt-get install dialog apt-utils

#RUN useradd -K MAIL_DIR=/dev/null -u 33 www-data

# Cache dir
RUN mkdir -p /var/lib/proxy && chown www-data /var/lib/proxy

ADD proxy /usr/local/bin/proxy
RUN chown www-data /usr/local/bin/proxy && \
    chmod +x /usr/local/bin/proxy

# Allow access to port 80
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y libcap2-bin
RUN setcap CAP_NET_BIND_SERVICE=+eip /usr/local/bin/proxy

USER www-data
# Todo: To put it somewhere: https://prometheus.io/

EXPOSE 80

CMD [ "/usr/local/bin/proxy" ]
