# Main site SPA routes (`/bili`)

After adding React Router, direct visits and refreshes on `https://taozhiyy.top/bili` need Nginx to fall back to `index.html`.

Inside the **main site** `server` / `location /` block (not `/blog/` or `/build/`):

```nginx
location / {
    try_files $uri $uri/ /index.html;
}
```

Then: `sudo nginx -t && sudo systemctl reload nginx`
