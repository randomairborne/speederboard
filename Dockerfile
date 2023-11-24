FROM alpine
ARG TARGETARCH

COPY /${TARGETARCH}-executables/speederboard /usr/bin/speederboard
COPY /templates/ /etc/speederboard/templates/
COPY /translations/ /etc/speederboard/translations/
COPY /assets/public/ /etc/speederboard/assets/public/

WORKDIR "/etc/speederboard/"
ENTRYPOINT "/usr/bin/speederboard"
