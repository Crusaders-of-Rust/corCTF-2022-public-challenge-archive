FROM python:3.8-slim-buster
WORKDIR /app
COPY website/ ./ 
CMD [ "python3", "-m", "http.server", "8001" ]
EXPOSE 8001
