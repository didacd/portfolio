# Deployment with GitHub Pages

The site is built as static Astro files and published by GitHub Actions. Pull
requests to `master` run lint and a production build; successful pushes to
`master` publish `dist` to GitHub Pages.

## Enable GitHub Pages

1. Push this repository and workflow to GitHub.
2. Open **Settings → Pages** in the repository.
3. Under **Build and deployment → Source**, select **GitHub Actions**.
4. Push a commit to `master` and confirm that the **GitHub Pages** workflow
   completes successfully in the Actions tab.

The workflow uses GitHub's Pages artifact and deployment actions. It needs no
deployment secrets, server, or manually managed HTTPS certificate.

## Custom domain

Configure the custom domain in GitHub before changing DNS:

1. In **Settings → Pages → Custom domain**, enter `didac.domenech.dev` and
   save it.
2. At the DNS provider, create a CNAME record with host `didac` pointing to
   `<your-github-username>.github.io`.
3. Wait for GitHub to verify the record, then enable **Enforce HTTPS** in the
   Pages settings.

GitHub Pages manages the certificate. A `CNAME` file is not needed when the
site is published by this custom Actions workflow.

## Rollback

Open **Actions → GitHub Pages**, select a previously successful workflow run,
and use **Re-run jobs** to redeploy that version. GitHub manages the deployed
artifact; there is no server-side release directory to maintain.

## Publishing content

Add or edit an MDX post under `src/content/posts/` or a project under
`src/content/projects/`, commit it, and merge the change into `master`.
Pull-request CI runs lint and a production build first. A successful merge
builds Pagefind into `dist/pagefind` and publishes the static files
automatically.
