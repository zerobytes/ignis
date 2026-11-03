# Development workflow

The canonical source and Git history are maintained at
**https://github.com/zerobytes/ignis**. All new engine, documentation, asset and
website development belongs to this repository.

## Working with Git

```sh
git clone https://github.com/zerobytes/ignis.git
cd ignis
git switch -c my-change
# Edit and run the checks appropriate to the change.
git add <changed-files>
git commit -m "Describe the change"
git push -u origin my-change
```

Open a pull request against `main`. Maintainers with direct push access use the
same public `origin`; an ordinary `git push` publishes the current branch to its
configured upstream. A commit records work locally; a push publishes it.

Do not send changes to a second repository. A retained private repository is a
historical archive, not another development branch or a required mirror. The
earlier snapshot exporter is retained as a historical utility, not the normal
development or release process. Do not merge the pre-publication private Git
history into this repository. Port any needed changes onto a public branch.

Local project context and reference inputs excluded by `.gitignore` stay on the
developer's machine. Do not force-add those files. Preserve upstream licenses
and include only assets that may be redistributed.

## Validation and versions

Follow [CONTRIBUTING.md](../CONTRIBUTING.md) and the
[quickstart](quickstart.md#verification) for validation. CI runs on public pushes
and pull requests. Use public commits and tags for releases; do not create a
parallel private version sequence.

## Website publication

Website source and generated `website/dist` files use this same Git history.
When authorized to publish the website:

1. Run `python3 website/deploy.py --prepare` to refresh the release manifest.
2. Commit the intended website files and push them to public `main` through the
   normal Git workflow.
3. Run `python3 website/deploy.py` to deploy and verify that exact static release
   on both Firebase Hosting and GitHub Pages.

The deployment command requires the static files to match committed public
`main`. It does not create remote-only source commits or push unrelated local
work. A mismatch stops publication and identifies the required commit/push step.
