# RISC-V programs

## Generate programs

```
cd program
```

```
docker run --platform linux/amd64 -it --rm -h playground \
          -e USER=$(id -u -n) \
          -e GROUP=$(id -g -n) \
          -e UID=$(id -u) \
          -e GID=$(id -g) \
          -v (pwd):/home/$(id -u -n) \
          -w /home/$(id -u -n) \
          diegonehab/playground:develop /bin/bash -c "./gen_machine_linux.sh"
```

```
docker run --platform linux/amd64 -it --rm -h playground \
          -e USER=$(id -u -n) \
          -e GROUP=$(id -g -n) \
          -e UID=$(id -u) \
          -e GID=$(id -g) \
          -v (pwd):/home/$(id -u -n) \
          -w /home/$(id -u -n) \
          diegonehab/playground:develop /bin/bash -c "./gen_machine_simple.sh"
```
