FROM php:7-cli-buster
# Using debian based image while compiling on vagrant

# set recommended PHP.ini settings
# excluding opcache due https://github.com/docker-library/wordpress/issues/407
# https://codex.wordpress.org/Editing_wp-config.php#Configure_Error_Logging
RUN { \
		echo 'error_reporting = 4339'; \
		echo 'display_errors = Off'; \
		echo 'display_startup_errors = Off'; \
		echo 'log_errors = On'; \
		echo 'error_log = /dev/stderr'; \
		echo 'log_errors_max_len = 1024'; \
		echo 'ignore_repeated_errors = On'; \
		echo 'ignore_repeated_source = Off'; \
		echo 'html_errors = Off'; \
	} > /usr/local/etc/php/conf.d/error-logging.ini

RUN apt-get update && apt-get install -y libzip-dev
RUN 	docker-php-ext-install \
		bcmath \
		exif \
		mysqli \
		zip

# Using mariadb-client instead of mysql-client as
# this is what buster provides
RUN apt-get install -y mariadb-client curl
RUN curl -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar \
    && chmod +x wp-cli.phar \
    && mv wp-cli.phar /usr/local/bin/wp

#ADD wp-cli-server /usr/local/bin/wp-cli-server
#RUN chown www-data:www-data /usr/local/bin/wp-cli-server && \
#    chmod +x /usr/local/bin/wp-cli-server

# Install ssh server
# https://www.cyberciti.biz/faq/ubuntu-linux-install-openssh-server/
# https://docs.docker.com/engine/examples/running_ssh_service/
RUN apt-get install -y apt-utils
RUN apt-get install -y openssh-server
RUN mkdir /var/run/sshd
# SSH login fix. Otherwise user is kicked off after login
RUN sed 's@session\s*required\s*pam_loginuid.so@session optional pam_loginuid.so@g' -i /etc/pam.d/sshd
# todo: Could only allow from host ip, see wp-dev for getting ip

RUN mkdir -p /var/www/html \
    && chown -R www-data:www-data /var/www/html

#USER www-data

# For logging in as www-data
# todo: There could possibly be a per site user setup,
# and generate keys for them.
# It has advantages in managing accessible files,
# but security wise a bit of a waste running as root,
# Podman might be piece of puzzle:
# https://podman.io/
RUN echo 'www-data:www-data' | chpasswd && \
	usermod --shell /bin/bash www-data


WORKDIR /var/www/html
RUN cat /etc/ssh/sshd_config

#CMD ["/usr/local/bin/wp-cli-server"]
EXPOSE 22
CMD ["/usr/sbin/sshd", "-D"]
