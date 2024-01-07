# crust

## Run Locally

To run some manual tests, we provided a testing podman environment. Prepared containers have a ssh server configured and you can connect to them from your local machine or directly between them. To setup the environment you need podman installed. The installation process differs between linux distros, so I am only going to show how to do it on debian/ubuntu (others can be found here https://podman.io/docs/installation).

Let's start with installing the podman itself.
```bash
sudo apt-get update && sudo apt-get -y install podman
```
The provided manager script can be used to run containers and retrieve their info. To start our podman containers we can use `start` command. You can use the `--build` flag to automatically prepare the image with ssh server running that is going to be used by our containers. Let's build the image using the manager script and run three containers.
```bash
sudo ./podman/manage_podman.py start --build --containers 3

01:21:54 PM   [INFO]   Starting 3 podman containers
01:21:54 PM   [INFO]   Starting container with image `ubuntu-ssh`
051a4b971a48514e45346e0df14de24c50b357884b7cb0ab134f7e408283f3c7
01:21:54 PM   [INFO]   Starting container with image `ubuntu-ssh`
9cf9512df36cc6972e00751a097a99a4c65f4094492dede6bf910e30fd1cbcfd
01:21:54 PM   [INFO]   Starting container with image `ubuntu-ssh`
b4f3a0b79558588f398c092328fbcc860461b4f07a8351efd4057b5de284595b
01:21:55 PM   [INFO]   Fetching podman container info
01:21:55 PM   [INFO]   Fetching all podman containers
01:21:55 PM   [INFO]   Fetching ip address of container `angry_colden`
01:21:55 PM   [INFO]   Container `angry_colden` - IP: `10.88.0.2`, USER: `test_user`, PASSWD: `1234`
01:21:55 PM   [INFO]   Fetching ip address of container `unruffled_lehmann`
01:21:55 PM   [INFO]   Container `unruffled_lehmann` - IP: `10.88.0.3`, USER: `test_user`, PASSWD: `1234`
01:21:55 PM   [INFO]   Fetching ip address of container `suspicious_lumiere`
01:21:55 PM   [INFO]   Container `suspicious_lumiere` - IP: `10.88.0.4`, USER: `test_user`, PASSWD: `1234`
```
To verify if the containers are really up and running, we can use
```bash
sudo podman ps

CONTAINER ID  IMAGE                        COMMAND            CREATED        STATUS            PORTS       NAMES
051a4b971a48  localhost/ubuntu-ssh:latest  /usr/sbin/sshd -D  3 minutes ago  Up 3 minutes ago              angry_colden
9cf9512df36c  localhost/ubuntu-ssh:latest  /usr/sbin/sshd -D  3 minutes ago  Up 3 minutes ago              unruffled_lehmann
b4f3a0b79558  localhost/ubuntu-ssh:latest  /usr/sbin/sshd -D  3 minutes ago  Up 3 minutes ago              suspicious_lumiere
```
Knowing that everything works, we can use ip addresses, usernames and passwords present in post-start logs, to connect to the container via ssh.
```bash
ssh test_user@10.88.0.2
```
If you lost the logs with provided ip addresses, you can always run `info` command to retrieve them again
```bash
sudo ./podman/manage_podman.py info

01:29:13 PM   [INFO]   Fetching podman container info
01:29:13 PM   [INFO]   Fetching all podman containers
01:29:13 PM   [INFO]   Fetching ip address of container `angry_colden`
01:29:13 PM   [INFO]   Container `angry_colden` - IP: `10.88.0.2`, USER: `test_user`, PASSWD: `1234`
01:29:13 PM   [INFO]   Fetching ip address of container `unruffled_lehmann`
01:29:13 PM   [INFO]   Container `unruffled_lehmann` - IP: `10.88.0.3`, USER: `test_user`, PASSWD: `1234`
01:29:13 PM   [INFO]   Fetching ip address of container `suspicious_lumiere`
01:29:13 PM   [INFO]   Container `suspicious_lumiere` - IP: `10.88.0.4`, USER: `test_user`, PASSWD: `1234`
```
When you are done testing, all you have to do is run the `stop` command.
```bash
sudo ./podman/manage_podman.py stop

01:30:05 PM   [INFO]   Stopping podman containers
01:30:05 PM   [INFO]   Fetching all podman containers
01:30:05 PM   [INFO]   Stopping container `angry_colden`
01:30:05 PM   [INFO]   Stopping container `unruffled_lehmann`
01:30:06 PM   [INFO]   Stopping container `suspicious_lumiere`
01:30:06 PM   [INFO]   Fetching podman container info
01:30:06 PM   [INFO]   Fetching all podman containers
01:30:07 PM   [WARNING]   No running containers found
```
If something is not clear, or is not working you can always check the `--help` flag for more context.
```bash
sudo ./podman/manage_podman.py --help
```
## Authors

- [@Leghart](https://gitlab.com/Leghart)
- [@WiktorNowak](https://gitlab.com/WiktorNowak)
