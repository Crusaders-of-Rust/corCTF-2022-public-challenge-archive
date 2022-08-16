FROM python:3.10-slim-bullseye AS app
RUN pip install pycryptodome --no-cache-dir

FROM pwn.red/jail:0.3.0
COPY --from=app / /srv
COPY --chmod=755 corruptedcurvesplus.py /srv/app/run
COPY flag.txt /srv/app/
ENV JAIL_TIME=1800 JAIL_PIDS=10 JAIL_MEM=100M JAIL_CPU=500
