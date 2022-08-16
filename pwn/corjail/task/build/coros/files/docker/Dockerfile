FROM debian:bullseye-slim

ARG USER=user

RUN apt update -y && apt install --no-install-recommends -y curl && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash $USER

RUN echo "export PS1='\033\[[01;31m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]# '"  >> /root/.bashrc
RUN echo "export PS1='\033\[[01;38;2;31;90;55m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]$ '" >> /home/$USER/.bashrc

RUN chmod -R 0700 /root
RUN chown -R $USER:$USER /home/$USER && chmod 0700 /home/$USER
