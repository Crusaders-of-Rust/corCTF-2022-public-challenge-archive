FROM debian:bullseye-slim

WORKDIR /app 
COPY app/requirements.txt /app/requirements.txt 
RUN apt update -y && apt install --no-install-recommends -y python3 python3-pip && rm -rf /var/lib/apt/lists/*
RUN python3 -m pip install -r requirements.txt  
COPY app /app  
ENTRYPOINT [ "python3" ] 
CMD [ "app.py" ] 
