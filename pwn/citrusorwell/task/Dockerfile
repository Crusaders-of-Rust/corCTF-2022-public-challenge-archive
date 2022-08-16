FROM ubuntu:20.04

RUN apt-get update && apt-get install -y socat

WORKDIR /app
COPY ./citrusorwell .
COPY ./snapshot_blob.bin .
RUN chmod a+xr citrusorwell snapshot_blob.bin
COPY ./flag.txt .

RUN useradd -m ctf
RUN chown -R ctf:ctf /app
USER ctf

EXPOSE 5000
CMD ["socat", "TCP-LISTEN:5000,reuseaddr,fork", "EXEC:/app/citrusorwell"]
