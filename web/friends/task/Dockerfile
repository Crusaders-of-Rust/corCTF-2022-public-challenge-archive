FROM node:18.1.0-bullseye-slim
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY index.js .
CMD ["node", "index.js"]
