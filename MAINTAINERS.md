# Maintainers

## Cutting a New Release

* Update the [CHANGELOG]:
  * Create a new branch to prepare the release.

  * Ensure the [CHANGELOG] is up to date. (See
    [below][changelog-refresh] for a suggested workflow.)

  * Retitle the "Unreleased" section to this release and create a fresh
    "Unreleased" section (see comments in the [CHANGELOG] for details).

    :bulb: Point releases (i.e., not patch releases) should also be
    given a name, taking the form `ADJECTIVE TREE`, incrementing
    alphabetically. This name should be decided amongst the team before
    the release.

  * Commit and merge (squash, if necessary) on green CI and peer
    approval.

  * Tag the merged commit with the release version, prefixed with a `v`
    (e.g., `v1.0.0`).

* [Draft a new release][draft-release] in GitHub.
  * Set the tag to that created in the previous step, now on `main`.
  * Set the release title to `Topiary v<RELEASE>`, or `Topiary
    v<RELEASE>: <NAME>` for point releases.
  * Copy-and-paste the [CHANGELOG] contents for this release
    into the description.
  * Publish.

* Publicise.

  :warning: Point releases, only. Don't publicise patch releases, unless
  there's a pressing need to do so (e.g., fix of a security
  vulnerability, etc.).

  * Announce the new version on Tweag's Twitter and other social network
    accounts, via someone with access.
  * Share amongst other social networks (e.g., Reddit, Hacker News,
    Mastodon, etc.), under personal accounts, at your discretion.

### Generating the PR List for the CHANGELOG

If the unreleased changes in the [CHANGELOG] have become stale, the list
of merged PRs can be fetched from:

    https://github.com/tweag/topiary/pulls?q=is:pr+base:main+merged:>YYYY-MM-DD

Replacing `YYYY-MM-DD` by the date of the last release.

If you have the GitHub CLI client, the following may be more convenient:

```bash
gh pr list -L 500 -B main -s merged \
           --json number,mergedAt,title,body \
| jq -r --argjson release "$(gh release view --json createdAt)" '
     reverse | .[] | select(.mergedAt > $release.createdAt) |
     ["# PR#\(.number): \(.title)", "*Merged: \(.mergedAt)*", "\(.body)\n"] |
     join("\n\n")'
```

:bulb: The `-L 500` is an arbitrary "large number" limit of PRs to
fetch, overriding the low default. As of writing, there's no way to set
this to "unlimited"; adjust as necessary.

<!-- Links -->
[changelog]: /changelog.md
[changelog-refresh]: #generating-the-pr-list-for-the-changelog
[draft-release]: https://github.com/tweag/topiary/releases/new
