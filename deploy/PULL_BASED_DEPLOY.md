# Taozhiyy pull-based deployment

This public repository must not run production deployment on a self-hosted GitHub runner. GitHub Actions now only validates builds on GitHub-hosted runners. The UCloud host deploys by pulling `master` itself from a dedicated server-side clone.

Build steps run as the locked-down `taozhiyy-deploy` user. Root is only used for fixed install, file sync, service restart, and Nginx reload operations.

## One-time server setup

Run this from the UCloud web console, VNC, or SSH session:

```bash
cd /tmp
curl -fsSL https://raw.githubusercontent.com/bistutzyy/taozhiyy/master/deploy/install-pull-deploy.sh -o install-pull-deploy.sh
sudo bash install-pull-deploy.sh
```

The installer:

1. Installs required build tools, Node.js 20, Go 1.22, Git, rsync, and Python 3.
2. Creates a system user named `taozhiyy-deploy`.
3. Clones `https://github.com/bistutzyy/taozhiyy.git` to `/opt/taozhiyy-source`.
4. Gives `/opt/taozhiyy-source` to `taozhiyy-deploy`.
5. Installs a root-owned deploy script at `/usr/local/sbin/taozhiyy-pull-deploy.sh`.
6. Ensures `/opt/acg-api/.env` is owned by `root:root` and mode `600`.
7. Installs `taozhiyy-pull-deploy.service` and `taozhiyy-pull-deploy.timer`.
8. Enables the timer and starts one deployment immediately.

## Runtime behavior

The timer runs every five minutes. `/usr/local/sbin/taozhiyy-pull-deploy.sh` fetches `origin/master`, compares it with `/var/lib/taozhiyy-pull-deploy/last-deployed-sha`, and exits if the current commit is already deployed.

When a new commit exists, the server:

1. Resets the dedicated clone to `origin/master` as `taozhiyy-deploy`.
2. Builds `acg-api`, `main`, `blog`, and `build` as `taozhiyy-deploy`.
3. Installs the backend binary and a fixed `acg-api.service`.
4. Keeps the existing production env file at `/opt/acg-api/.env` with `root:root` ownership and `600` permissions.
5. Backs up `/var/www/taozhiyy`.
6. Syncs the new static files into `/var/www/taozhiyy`.
7. Reloads or restarts Nginx and runs health checks.

## Secrets

Production secrets live only on the server, primarily in `/opt/acg-api/.env`. GitHub Actions no longer reads or writes production secrets for deployment.

The deploy script does not read `/opt/acg-api/.env`. The `acg-api` process loads it in-process when systemd starts the service.

If a new backend feature needs a secret, add it manually to `/opt/acg-api/.env` on the server and restart or force the pull deploy:

```bash
sudo FORCE_DEPLOY=1 /usr/local/sbin/taozhiyy-pull-deploy.sh
```

## Operations

Check timer status:

```bash
sudo systemctl status taozhiyy-pull-deploy.timer --no-pager
```

Check the latest deploy run:

```bash
sudo journalctl -u taozhiyy-pull-deploy.service -n 200 --no-pager
```

Force a redeploy of the current `master` commit:

```bash
sudo FORCE_DEPLOY=1 /usr/local/sbin/taozhiyy-pull-deploy.sh
```

Pause automatic deploys:

```bash
sudo systemctl disable --now taozhiyy-pull-deploy.timer
```
