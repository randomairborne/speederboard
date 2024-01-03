FROM alpine AS compressor

RUN apk add zstd brotli gzip

COPY /assets/public/ /assets/public/

RUN find /assets/public/ -type f -exec gzip -k9 '{}' \; -exec brotli -k9 '{}' \; -exec zstd -qk19 '{}' \;

FROM alpine
ARG TARGETARCH

COPY /${TARGETARCH}-executables/speederboard /usr/bin/speederboard
COPY /templates/ /var/www/speederboard/templates/
COPY /translations/ /var/www/speederboard/translations/
COPY --from=compressor /assets/public/ /var/www/speederboard/static/

ENV ASSET_DIR="/var/www/speederboard/static/"
ENV TEMPLATE_DIR="/var/www/speederboard/templates/"
ENV TRANSLATION_DIR="/var/www/speederboard/translations/"

ENTRYPOINT "/usr/bin/speederboard"
