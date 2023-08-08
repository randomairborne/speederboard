FROM alpine
ARG TARGETARCH

COPY /${TARGETARCH}-executables/speederboard /usr/bin/
COPY /public/ /etc/speederboard/public/
COPY /templates/ /etc/speederboard/templates/

WORKDIR "/etc/speederboard/"
ENTRYPOINT "/usr/bin/speederboard"
