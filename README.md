# Current development progress

Since my energy is limited, I mainly focus on fixing bugs. New features will not be updated for the time being.

# To do

- web ui support i18n.
- category and post support upload cover.
  - [ ] web ui
  - [x] server
- add follow system.
- support smtp.
- support delete actions.
- write tests

# Note before deployment

- **If using cloudflare or similar service proxy dns. You must enable `forwarded_ip` in `config.toml`, otherwise API access will be restricted by proxy IP, causing all users to be restricted from access together.**

# Docker Compose

```
version: "3.9"
services:
  server:
    image: jinker25/fofo-server:main
    container_name: fofo-server
    user: 1000:1000
    ports:
      - "6688:6688"
    restart: unless-stopped
    volumes:
      - "<your_data_folder_path>:/data"

  web:
    image: jinker25/fofo-web:main
    container_name: fofo-web
    user: 1000:1000
    ports:
      - "6687:6687"
    environment:
      - NUXT_BASE_URL="http://server:6688"
      - NUXT_PUBLIC_BASE_URL=""
      - NUXT_PUBLIC_FORUM_NAME="Fofo"
      - NUXT_PUBLIC_LIMIT_DATA_ANY=20
      - NUXT_PUBLIC_LIMIT_DATA_COMMENTS=10
      - NUXT_PUBLIC_DEFAULT_DISTINCT=false
    restart: unless-stopped
```

`sudo docker compose up -d` to start the server and web.

# Docker + Serverless Frontend

## Start the server only

```
version: "3.9"
services:
  server:
    image: jinker25/fofo-server:main
    container_name: fofo-server
    user: 1000:1000
    ports:
      - "6688:6688"
    restart: unless-stopped
    volumes:
      - "<your_data_folder_path>:/data"
```

`sudo docker compose up -d` to start the server only.

You can read the this [article](https://nuxt.com/docs/guide/concepts/rendering#edge-side-rendering) about the `Edge-Side Rendering (ESR)`.

## Configure the cloudflare pages.

I'm using the cloudflare pages.

1. fork the `fofo` github respository.
2. In the cloudflare pages, connect your github account to access your repository. Choose the `fofo`.
3. Set up builds and deployments
   - Framework preset choose `Nuxt.js`.
   - Root directory (advanced) write `web` (no slash) because our web project is under the `/web` directory.
   - Environment variables (advanced). Set the config of web you want. See the "fofo-web" section in docker-compose.yml above.
4. Click `Save and Deploy` now!
5. **If enabled cloudflare proxy dns. You must enable `forwarded_ip` in `config.toml`, otherwise API access will be restricted by proxy IP, causing all users to be restricted from access together.**

# Configuration

create the `config.toml` under the `--data-path` directory.

```toml
# Buffer size. Sqlite maximum bulk insert is 999.
buffer_size = 999
# Request pagination maximum limit.
fetch_limit = 30
# All task trigger interval in millisecond.
task_trigger_ms = 500
# Users logined active duration in day.
auth_active_days = 100
# resources expiry duration in second.
resource_expiry_seconds = 3600
# temporary resource expiry duration in second.
temporary_expiry_seconds = 60
# users get presign url expiry duration in second.
presign_expiry_seconds = 8
# All check task trigger interval in millisecond.
check_task_interval_seconds = 3600
# The maximum capacity of entries that the cache can hold.
cache_max_capacity = 1000
# ttl in second.
ttl_seconds = 5
# tti in second.
tti_seconds = 5
# Post editable duration in second.
editable_seconds = 30 * 60
# Post and comment top index maximum, ignore admin.
top_index_max = 9
# Auto fetch the post cover from article if post created cover is empty.
auto_fetch_post_cover = true
# User can upload the post cover or not.
custom_post_cover_supported = false
# User can register or not.
open_register = true
# Use forwarded ip instead peer ip. It use header `x-forwarded-for` to get the ip.
forwarded_ip = false
# The bypass key for rate limit. If request with header `x-bypass-key` equals to this key will bypass it. Default is none.
bypass_key = ""
# Console log level.
log_level = "info"
# Image format. Example, using when generate captcha. Go to https://docs.rs/image/latest/image/enum.ImageFormat.html see more.
image_format = "jpeg"

# Local storage service config. (If S3 is disabled)
[local]
# Public url of local storage. For example, getting the verification will return the image url. This is the base URL.
public_url = ""
# Max bytes of client upload.
max_bytes = 2097152

# Use API compatible with the Amazon S3 cloud storage service. Defaults to None, if defined it means enabled.
[s3]
# Bucket of Amazon S3 cloud storage service.
bucket = ""
# Region of Amazon S3 cloud storage service.
region = ""
# Endpoint of Amazon S3 cloud storage service.
endpoint = ""
# Endpoint of Amazon S3 cloud storage service.
public_url = ""
# Access Key ID of Amazon S3 cloud storage service.
access_key = ""
# Secret Access Key of Amazon S3 cloud storage service.
secret_key = ""
```
