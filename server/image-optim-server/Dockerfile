FROM colthreepv/docker-image_optim
# https://github.com/colthreepv/docker-image_optim/blob/master/Dockerfile
# https://github.com/nwtn/image-resize-tests/tree/optimization

# alpine
RUN echo "**** install Python ****" && \
    apk add --no-cache python3 && \
    if [ ! -e /usr/bin/python ]; then ln -sf python3 /usr/bin/python ; fi && \
    \
    echo "**** install pip ****" && \
    python3 -m ensurepip && \
    rm -r /usr/lib/python*/ensurepip && \
    pip3 install --no-cache --upgrade pip setuptools wheel && \
    if [ ! -e /usr/bin/pip ]; then ln -s pip3 /usr/bin/pip ; fi

# Go with www-data user as it appears in
# wordpress container with id 33
# Adding to vagrant group for dev server
RUN apk --no-cache add shadow && \
    useradd -K MAIL_DIR=/dev/null -u 33 --non-unique www-data

# For dev server adding to vagrant group
RUN groupadd --gid 1000 --non-unique vagrant && usermod -a -G vagrant www-data

USER www-data

ADD server.py /opt/server.py

# todo: Check where this is placed, it might have ended up in uploads
ADD config.yml /opt/config.yml

EXPOSE 8971
# -u for unbuffered makes prints show up in logs
ENTRYPOINT [ "python3", "-u", "/opt/server.py" ]
