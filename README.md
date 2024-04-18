# To do
- web ui support i18n.
- category and post support upload cover.
    - [ ] web ui
    - [x] server
- add follow system.
- support smtp.
- support delete actions.
- write tests

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