FROM archlinux/base:latest


RUN rm /etc/pacman.d/mirrorlist
RUN echo "Server = http://mirrors.evowise.com/archlinux/\$repo/os/\$arch" >> /etc/pacman.d/mirrorlist

RUN  pacman-db-upgrade; pacman -Syy --noconfirm glibc zeromq postgresql-libs dnsutils iproute2; pacman -Scc --noconfirm

ARG app_file
ADD $app_file /app

ENTRYPOINT ["/app"]