FROM alpine
ARG TARGETARCH

COPY /${TARGETARCH}-executables/speederboard /usr/bin/
COPY /templates/ /etc/speederboard/templates/
COPY /translations/ /etc/speederboard/translations/

WORKDIR "/etc/speederboard/"
ENTRYPOINT "/usr/bin/speederboard"
