FROM node:18.6.0-bullseye-slim
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY src .
CMD ["node", "submit.js"]
