FROM wordpress:5
# vagrant group is 1000, adding to
# enable access to mounted volumes
#RUN groupadd --gid 1000 vagrant && usermod -a -G vagrant www-data

COPY custom-entrypoint.sh /usr/local/bin
RUN chmod +x /usr/local/bin/custom-entrypoint.sh

ENTRYPOINT ["custom-entrypoint.sh"]
CMD ["apache2-foreground"]
