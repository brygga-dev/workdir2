FROM ubuntu:xenial

# https://cloudogu.com/en/blog/backup-and-restore-with-restic

RUN apt-get update && apt-get install -y curl
# todo: Check https://github.com/aptible/supercronic/
ENV SUPERCRONIC_URL=https://github.com/aptible/supercronic/releases/download/v0.1.9/supercronic-linux-amd64 \
    SUPERCRONIC=supercronic-linux-amd64 \
    SUPERCRONIC_SHA1SUM=5ddf8ea26b56d4a7ff6faecdd8966610d5cb9d85

RUN curl -fsSLO "$SUPERCRONIC_URL" \
 && echo "${SUPERCRONIC_SHA1SUM}  ${SUPERCRONIC}" | sha1sum -c - \
 && chmod +x "$SUPERCRONIC" \
 && mv "$SUPERCRONIC" "/usr/local/bin/${SUPERCRONIC}" \
 && ln -s "/usr/local/bin/${SUPERCRONIC}" /usr/local/bin/supercronic
# https://github.com/camilb/kube-mysqldump-cron
# https://github.com/alexanderschnitzler/docker-mysqldump



#RUN echo "wordpress" | mysql_config_editor set --login-path=local --host=localhost --user=wordpress --password

# For dev machine I think vagrant needs www-data
#RUN apk --no-cache add shadow && \
#    useradd -K MAIL_DIR=/dev/null -u 1000 vagrant && \
#    usermod -a -G 33 vagrant


RUN apt-get install -y \
    apt-utils \
    mysql-client \
    git

# Move files
RUN mkdir -p /var/lib/docker-backup/repo && \
    chown -R www-data:www-data /var/lib/docker-backup

COPY cron-job.sh /
RUN chown www-data:www-data /cron-job.sh && chmod +x /cron-job.sh

COPY restore.sh /opt/
RUN chown www-data:www-data /opt/restore.sh && chmod +x /opt/restore.sh

COPY lib.sh /opt/
RUN chown www-data:www-data /opt/lib.sh && chmod +x /opt/lib.sh

COPY crontab /etc/backup-crontab
RUN chown www-data:www-data /etc/backup-crontab && \
    chmod 600 /etc/backup-crontab

# Run as www-data
USER www-data
CMD [ "supercronic",  "/etc/backup-crontab" ]
