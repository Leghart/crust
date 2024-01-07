FROM ubuntu:latest

RUN apt update && \
    apt install -y sudo openssh-server && \
    useradd -rm -d /home/ubuntu -s /bin/bash -g root -G sudo -u 1000 test_user && \
    echo "test_user:1234" | chpasswd && \
    service ssh start

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D"]
