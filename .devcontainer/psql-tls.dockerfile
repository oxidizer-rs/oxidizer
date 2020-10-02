FROM postgres:13

COPY --chown=postgres:postgres dev-pki/server.key /var/lib/postgresql/server.key
COPY --chown=postgres:postgres dev-pki/server.cert /var/lib/postgresql/server.cert
RUN chmod 600 /var/lib/postgresql/server.key
