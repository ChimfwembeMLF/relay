FROM node:22-alpine AS deps
WORKDIR /app
COPY package.json package-lock.json* ./
RUN if [ -f package-lock.json ]; then npm ci --omit=dev; else npm install --omit=dev; fi

FROM node:22-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
RUN addgroup -S relay && adduser -S relay -G relay
COPY --from=deps /app/node_modules ./node_modules
COPY package.json ./
COPY src ./src
USER relay
CMD ["npm", "start"]
