#!/usr/bin/python3
import logging
import subprocess
import sys
from argparse import ArgumentParser, Namespace
from pathlib import Path
from typing import Final

CURRENT_DIR_PATH: Final = Path(__file__).parent.absolute()
IMAGE_NAME: Final = "ubuntu-ssh"
USER: Final = "test_user"
PASSWD: Final = "1234"

logging.basicConfig(format="%(asctime)s   [%(levelname)s]   %(message)s", level=logging.INFO, datefmt="%I:%M:%S %p")
logger = logging.getLogger(__name__)


def _parse_args() -> Namespace:
    parser = ArgumentParser(
        prog="./setup_podman.py",
        description="Builds and runs podman pod with `n` containers for ssh testing.",
        epilog="Authors: @WiktorNowak, @Leghart",
    )
    subparsers = parser.add_subparsers(required=True, dest="command")
    run_parser = subparsers.add_parser("start", help="Creates image and runs podman containers.")
    subparsers.add_parser("info", help="Prints info about currently running containers.")
    subparsers.add_parser("stop", help="Stops all currently running containers.")

    run_parser.add_argument("--build", action="store_true", help="If set, rebuilds the image used for ssh containers.")
    run_parser.add_argument(
        "--containers", default=1, type=int, help="The amount of containers that are going to be created."
    )
    return parser.parse_args()


def _get_all_running_container_names() -> list[str]:
    logger.info("Fetching all podman containers")
    result = subprocess.run(("sudo", "podman", "ps", "--format", "{{.Names}}"), capture_output=True)

    if result.stderr:
        logger.error("Something went wrong when reading container names - %s", result.stderr)
        return []

    return result.stdout.decode("utf-8").split("\n")[:-1]


def _get_container_ip_address(container_name: str) -> str:
    logger.info("Fetching ip address of container `%s`", container_name)
    result = subprocess.run(
        (
            "sudo",
            "podman",
            "inspect",
            "--format",
            "{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
            container_name,
        ),
        capture_output=True,
    )

    if result.stderr:
        logger.error("Something went wrong when container ip address - %s", result.stderr)
        return "0.0.0.0"

    return result.stdout.decode("utf-8")


def print_podman_info() -> None:
    logger.info("Fetching podman container info")
    containers = _get_all_running_container_names()

    if not containers:
        logger.warning("No running containers found")
        return None

    for container_name in containers:
        ip = _get_container_ip_address(container_name)
        logger.info("Container `%s` - IP: `%s`, USER: `%s`, PASSWD: `%s`", container_name, ip.strip(), USER, PASSWD)


def build_podman_image() -> None:
    logger.info("Building image `%s`", IMAGE_NAME)
    subprocess.run(("sudo", "podman", "build", "-t", IMAGE_NAME, "-f", CURRENT_DIR_PATH / "Dockerfile"))


def start_podman_containers(containers: int) -> None:
    logger.info("Starting %d podman containers", containers)

    for _ in range(containers):
        logger.info("Starting container with image `%s`", IMAGE_NAME)
        subprocess.run(("sudo", "podman", "run", "-dt", IMAGE_NAME))


def stop_podman_containers() -> None:
    logger.info("Stopping podman containers")

    containers = _get_all_running_container_names()
    for container_name in containers:
        logger.info("Stopping container `%s`", container_name)
        subprocess.run(("sudo", "podman", "stop", container_name))


def main() -> int:
    namespace = _parse_args()

    match namespace.command:
        case "start":
            if namespace.build is True:
                build_podman_image()
            start_podman_containers(namespace.containers)
        case "stop":
            stop_podman_containers()

    print_podman_info()

    return 0


if __name__ == "__main__":
    sys.exit(main())
