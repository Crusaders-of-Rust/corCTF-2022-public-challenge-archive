FROM ubuntu 

RUN apt-get update && apt-get install -y \
	curl \
	imagemagick \
	python3-pip \
	&& rm -rf /var/lib/apt/lists/*
RUN pip3 install flask gunicorn

COPY server.py /app/
COPY wsgi.py /app/
COPY fe /app/fe/
COPY img /app/img/
COPY flag.txt /flag.txt

EXPOSE 5000
WORKDIR /app
USER www-data

CMD ["gunicorn", "--workers=8", "--bind", "0.0.0.0:5000", "wsgi:app"]
