FROM --platform=$BUILDPLATFORM node:23-slim AS node-source

FROM --platform=$BUILDPLATFORM denoland/deno:2.1.4 AS deno-builder

# Get NodeJS
COPY --from=node-source /usr/local/bin /usr/local/bin
# Get npm
COPY --from=node-source /usr/local/lib/node_modules /usr/local/lib/node_modules

RUN npm install -g npm

COPY sdks/typescript /app/sdks/typescript

RUN deno cache sdks/typescript/*/scripts/build_npm.ts

WORKDIR /app/sdks/typescript/hyveos-sdk

RUN deno run -A scripts/build_npm.ts 1.0.0 

WORKDIR /app/sdks/typescript/hyveos-web

RUN deno run -A scripts/build_npm.ts 1.0.0

FROM --platform=$BUILDPLATFORM node:23-slim AS ui-builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

COPY --from=deno-builder /app/sdks/typescript/hyveos-sdk/npm /app/sdks/typescript/hyveos-sdk/npm
COPY --from=deno-builder /app/sdks/typescript/hyveos-web/npm /app/sdks/typescript/hyveos-web/npm

COPY ui /app/ui
WORKDIR /app/ui


FROM ui-builder AS prod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --forzen-lockfile

FROM ui-builder AS ui-build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build


FROM busybox:musl AS final

COPY --from=ui-build /app/ui/build /www

ENV PORT=8080

EXPOSE $PORT

CMD ["sh", "-c", "httpd -f -p ${PORT} -h /www"]


LABEL org.opencontainers.image.title="hyveOS Debug UI"
LABEL org.opencontainers.image.documentation="https://docs.p2p.industries"
LABEL org.opencontainers.image.licenses="GPL-3.0-only"
