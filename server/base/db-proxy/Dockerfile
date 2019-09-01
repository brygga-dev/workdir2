FROM ubuntu:bionic
RUN apt-get update

# To not run as root
RUN useradd -K MAIL_DIR=/dev/null run-user

ADD db-proxy /usr/local/bin/db-proxy

RUN chown run-user:run-user /usr/local/bin/db-proxy && \
    chmod +x /usr/local/bin/db-proxy

USER run-user

EXPOSE 3308

ENTRYPOINT ["/usr/local/bin/db-proxy"]