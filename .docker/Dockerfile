FROM docker/sandbox-templates:opencode

USER root
COPY .docker/entrypoint.sh /usr/local/bin/sandbox-entrypoint.sh
RUN chmod +x /usr/local/bin/sandbox-entrypoint.sh

USER agent
ENTRYPOINT ["/usr/local/bin/sandbox-entrypoint.sh"]
