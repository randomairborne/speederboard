FROM alpine AS compressor

COPY /assets/public/ /assets/public/

RUN find /assets/public/ -type f -exec gzip -k9 '{}' \; -exec brotli -k9 '{}' \; -exec zstd -qk19 '{}' \; 

FROM alpine
ARG TARGETARCH

COPY /${TARGETARCH}-executables/speederboard /usr/bin/speederboard
COPY /templates/ /etc/speederboard/templates/
COPY /translations/ /etc/speederboard/translations/
COPY --from=compressor /assets/public/ /etc/speederboard/assets/public/

WORKDIR "/etc/speederboard/"
ENTRYPOINT "/usr/bin/speederboard"
