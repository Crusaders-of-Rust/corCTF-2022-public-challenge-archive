FROM pwn.red/jail:0.3.0
COPY --from=python:slim / /srv
COPY --chmod=755 luckyguess.py /srv/app/run
COPY flag.txt /srv/app/
ENV JAIL_TIME=180 JAIL_PIDS=10 JAIL_MEM=30M